#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Embassy STM32F4 LED Blink started!");

    // PC13: onboard LED (active low)
    let mut led = Output::new(p.PC13, Level::High, Speed::Low);

    loop {
        led.toggle();
        info!("LED toggled");
        Timer::after_millis(500).await;
    }
}
