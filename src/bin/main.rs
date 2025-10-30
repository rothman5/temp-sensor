#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::mem::take;

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use esp_backtrace as _;
use esp_hal::Async;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::timer::timg::TimerGroup;
use log::{error, info};
use temp_sensor::mcp9808::{Address, MCP9808};

extern crate alloc;
use alloc::vec::Vec;
use temp_sensor::registers::resolution::TempRes;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // Initialize logging
    esp_println::logger::init_logger_from_env();

    // Configure the HAL
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());

    // Initialize the HAL
    let peripherals = esp_hal::init(config);

    // Set up the heap allocator. Used for async tasks and other dynamic allocations.
    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 98767);

    // Start the embassy executor
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("HAL and Embassy initialized.");

    // Initialize Wi-Fi/BLE controller
    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO21)
        .with_scl(peripherals.GPIO22)
        .into_async();

    spawner.spawn(acquisition_task(i2c)).unwrap();

    loop {
        info!("Emulating monitoring Wi-Fi connection...");
        Timer::after(Duration::from_secs(60)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.1/examples/src/bin
}

enum AcquisitionState {
    Idle,
    RxData,
    TxData,
}

struct Measurement {
    pub timestamp: Instant,
    pub in_temp_c: f32,
    pub out_temp_c: f32,
}

async fn identify(s: &MCP9808, i2c: &mut I2c<'static, Async>) -> Option<u8> {
    let dev_info = s.get_device_info(i2c).await.ok()?;
    Some(dev_info.get_device_id())
}

async fn measure(s1: &MCP9808, s2: &MCP9808, i2c: &mut I2c<'static, Async>) -> Option<Measurement> {
    let t1 = s1.get_temp(i2c).await.ok()?;
    let t2 = s2.get_temp(i2c).await.ok()?;
    Some(Measurement {
        timestamp: Instant::now(),
        in_temp_c: t1.get_celsius(TempRes::Deg_0_0625C),
        out_temp_c: t2.get_celsius(TempRes::Deg_0_0625C),
    })
}

async fn transmit(batch: Vec<Measurement>) -> Result<(), ()> {
    if batch.is_empty() {
        return Err(());
    }

    info!("Transmitting batch of {} measurements...", batch.len());
    for (i, m) in batch.iter().enumerate() {
        info!(
            "Measurement #{}: Time: {:?}, Inside: {:.2} °C, Outside: {:.2} °C",
            i + 1,
            m.timestamp,
            m.in_temp_c,
            m.out_temp_c
        );
    }

    Ok(())
}

#[embassy_executor::task]
async fn acquisition_task(mut i2c: I2c<'static, Async>) {
    const SAMPLE_TIME_MS: u64 = 150_000;
    const BATCH_SIZE: usize = 16;

    let mut state = AcquisitionState::Idle;
    let mut batch: Vec<Measurement> = Vec::with_capacity(BATCH_SIZE);

    let in_sensor = MCP9808::new(Address::Default);
    let out_sensor = MCP9808::new(Address::Alternate {
        bit2: true,
        bit1: false,
        bit0: false,
    });

    loop {
        match state {
            AcquisitionState::Idle => {
                // Wait for the next acquisition cycle
                Timer::after_millis(SAMPLE_TIME_MS).await;
                state = AcquisitionState::RxData;
            }

            AcquisitionState::RxData => {
                // Identify the inside sensor
                let in_id = match identify(&in_sensor, &mut i2c).await {
                    Some(id) => id,
                    None => {
                        error!("Failed to identify inside sensor. Skipping measurement...");
                        state = AcquisitionState::Idle;
                        continue;
                    }
                };

                // Identify the outside sensor
                let out_id = match identify(&out_sensor, &mut i2c).await {
                    Some(id) => id,
                    None => {
                        error!("Failed to identify outside sensor. Skipping measurement...");
                        state = AcquisitionState::Idle;
                        continue;
                    }
                };

                // Acquire measurement
                let meas = match measure(&in_sensor, &out_sensor, &mut i2c).await {
                    Some(m) => m,
                    None => {
                        error!("Failed to acquire measurement. Skipping...");
                        state = AcquisitionState::Idle;
                        continue;
                    }
                };

                info!("Sample acquired [batched {}/{}]", batch.len(), BATCH_SIZE);
                info!("Inside (id={:X}): {:.2} °C", in_id, meas.in_temp_c);
                info!("Outside (id={:X}): {:.2} °C", out_id, meas.out_temp_c);

                // Store measurement
                batch.push(meas);

                // Check if batch is full
                if batch.len() < BATCH_SIZE {
                    // Continue acquiring data until the batch is full
                    state = AcquisitionState::Idle;
                } else {
                    // Batch is full, proceed to transmission
                    state = AcquisitionState::TxData;
                }
            }

            AcquisitionState::TxData => {
                // Take the batch, leaving an empty vector in its place to continue collecting new data
                let batch_to_send = take(&mut batch);

                // Transmit batched data
                if let Err(_) = transmit(batch_to_send).await {
                    error!("Failed to transmit batched measurements.");
                    // Optionally, could re-push batch_to_send back into batch for retry
                } else {
                    info!("Batched measurements transmitted successfully.");
                }

                // Return to idle state for next acquisition cycle
                state = AcquisitionState::Idle;
            }
        }
    }
}
