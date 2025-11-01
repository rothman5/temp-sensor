#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Instant, Timer};
use esp_backtrace as _;
use esp_hal::Async;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::timer::timg::TimerGroup;
use log::{error, info};
use temp_sensor::mcp9808::{Address, MCP9808};
use temp_sensor::measurement::Measurement;
use temp_sensor::registers::resolution::TempRes;

// This creates a default app-descriptor required by the esp-idf bootloader.
esp_bootloader_esp_idf::esp_app_desc!();

const SAMPLE_TIME: u64 = 150;
const BATCH_SIZE: usize = 6;

static BATCH_CHANNEL: Channel<CriticalSectionRawMutex, Measurement, BATCH_SIZE> = Channel::new();

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

    spawner.spawn(rx_task(i2c)).unwrap();
    spawner.spawn(tx_task()).unwrap();

    loop {
        // TODO: Monitor Wi-Fi connection and handle reconnections
        info!("Emulating monitoring Wi-Fi connection...");
        Timer::after(Duration::from_secs(60)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.1/examples/src/bin
}

#[embassy_executor::task]
async fn rx_task(mut i2c: I2c<'static, Async>) {
    info!("Starting measurement task...");

    let in_sensor = MCP9808::new(Address::Default);
    let out_sensor = MCP9808::new(Address::Alternate {
        bit2: true,
        bit1: false,
        bit0: false,
    });

    info!("Identifying sensors...");

    // Identify the inside sensor
    let in_id = match identify(&in_sensor, &mut i2c).await {
        Some(id) => {
            info!("Found inside sensor [id={:X}]", id);
            id
        }
        None => {
            error!("Failed to identify inside sensor. Is it connected?");
            return;
        }
    };

    // Identify the outside sensor
    let out_id = match identify(&out_sensor, &mut i2c).await {
        Some(id) => {
            info!("Found outside sensor [id={:X}]", id);
            id
        }
        None => {
            error!("Failed to identify outside sensor. Is it connected?");
            return;
        }
    };

    let mut batched: u8 = 0;

    loop {
        // Acquire measurement
        let meas = match measure(&in_sensor, &out_sensor, &mut i2c).await {
            Some(m) => m,
            None => {
                error!("Failed to acquire measurement. Skipping...");
                continue;
            }
        };

        // Store measurement in channel
        BATCH_CHANNEL.send(meas).await;
        batched += 1;

        info!("Batched sample [{}/{}]", batched, BATCH_SIZE);
        info!("Inside (id={:X}): {:.2} °C", in_id, meas.in_temp_c);
        info!("Outside (id={:X}): {:.2} °C", out_id, meas.out_temp_c);

        if batched >= BATCH_SIZE as u8 {
            batched = 0;
        }

        // Wait for the next acquisition cycle
        Timer::after_secs(SAMPLE_TIME).await;
    }
}

#[embassy_executor::task]
async fn tx_task() {
    info!("Starting transmission task...");

    let mut batch: [Measurement; BATCH_SIZE] = [Measurement::default(); BATCH_SIZE];

    loop {
        info!("Waiting for batch to fill...");

        // Collect the batch from the channel, blocking until full
        for i in 0..BATCH_SIZE {
            batch[i] = BATCH_CHANNEL.receive().await;
        }

        info!("Batch full. Transmitting...");

        // Transmit batched data
        if let Err(_) = transmit(&batch).await {
            error!("Failed to transmit batched measurements.");
        } else {
            info!("Batched measurements transmitted successfully.");
        }
    }
}

async fn identify(sen: &MCP9808, i2c: &mut I2c<'static, Async>) -> Option<u8> {
    let dev_info = sen.get_device_info(i2c).await.ok()?;
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

async fn transmit(batch: &[Measurement]) -> Result<(), ()> {
    // Check for empty batch
    if batch.is_empty() {
        return Err(());
    }

    info!("Transmitting batch of {} measurements...", batch.len());
    for (i, m) in batch.iter().enumerate() {
        info!("Measurement #{}: {}", i + 1, m.to_string());

        // TODO: TCP transmission logic would go here
    }

    Ok(())
}
