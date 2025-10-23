#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::cell::RefCell;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::timer::timg::TimerGroup;
use log::info;
use temp_sensor::mcp9808::{Address, MCP9808};
use temp_sensor::registers::device_id::DeviceId;
use temp_sensor::registers::resolution::TempRes;
use temp_sensor::registers::temperature::Temperature;

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 0.6.0

    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 98767);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    // TODO: Spawn some tasks
    let _ = spawner;

    let i2c = I2c::new(peripherals.I2C0, Config::default())
        .unwrap()
        .with_sda(peripherals.GPIO21)
        .with_scl(peripherals.GPIO22);

    let i2c_cell = RefCell::new(i2c);

    let mut mcp9808_1 = MCP9808::new(RefCellDevice::new(&i2c_cell), Address::Default);
    let mut mcp9808_2 = MCP9808::new(
        RefCellDevice::new(&i2c_cell),
        Address::Alternate {
            bit2: true,
            bit1: false,
            bit0: false,
        },
    );

    loop {
        let info_1 = mcp9808_1.get_device_info().unwrap();
        let info_2 = mcp9808_2.get_device_info().unwrap();

        let temp_1 = mcp9808_1.get_temperature().unwrap();
        let temp_2 = mcp9808_2.get_temperature().unwrap();

        info!(
            "MCP9808 #1 [id: {:X}] - Temperature: {:.2} °C",
            info_1.get_device_id(),
            temp_1.get_celsius(TempRes::Deg_0_0625C)
        );
        info!(
            "MCP9808 #2 [id: {:X}] - Temperature: {:.2} °C",
            info_2.get_device_id(),
            temp_2.get_celsius(TempRes::Deg_0_0625C)
        );

        Timer::after(Duration::from_secs(3)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.1/examples/src/bin
}
