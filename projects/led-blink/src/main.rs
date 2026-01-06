#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // Keep SWD debug pins (PA13/PA14) enabled during sleep.
    // Without this, probe-rs cannot reconnect after the first flash,
    // resulting in "JtagNoDeviceConnected" errors.
    config.enable_debug_during_sleep = true;
    let p = embassy_stm32::init(config);
    info!("Embassy STM32F4 LED Blink started!");

    // PC13: onboard LED (active low)
    let mut led = Output::new(p.PC13, Level::High, Speed::Low);

    loop {
        led.toggle();
        info!("LED toggled");
        Timer::after_millis(1000).await;
    }
}
