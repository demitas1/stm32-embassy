//! RGB LED Rainbow Effect using Embassy on STM32F411CE (Black Pill)
//!
//! GPIO Configuration (active high, common cathode RGB LED):
//!   - PB6 = Red   (TIM4_CH1)
//!   - PB7 = Green (TIM4_CH2)
//!   - PB8 = Blue  (TIM4_CH3)
//!   - PC13 = Status LED (active low)

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// Convert HSV to RGB
///
/// # Arguments
/// * `hue` - Hue value (0-359)
/// * `sat` - Saturation (0-255)
/// * `val` - Value/Brightness (0-255)
///
/// # Returns
/// (r, g, b) tuple with values 0-255
fn hsv_to_rgb(hue: u16, sat: u8, val: u8) -> (u8, u8, u8) {
    if sat == 0 {
        return (val, val, val);
    }

    let region = hue / 60;
    let remainder = ((hue % 60) as u16 * 255 / 60) as u8;

    let p = ((val as u16 * (255 - sat as u16)) / 255) as u8;
    let q = ((val as u16 * (255 - (sat as u16 * remainder as u16) / 255)) / 255) as u8;
    let t = ((val as u16 * (255 - (sat as u16 * (255 - remainder as u16)) / 255)) / 255) as u8;

    match region {
        0 => (val, t, p),
        1 => (q, val, p),
        2 => (p, val, t),
        3 => (p, q, val),
        4 => (t, p, val),
        _ => (val, p, q),
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    // NOTE: enable_debug_during_sleep sets DBGMCU_CR=0x7 (DBG_SLEEP|DBG_STOP|DBG_STANDBY),
    // but this does NOT resolve JtagNoDeviceConnected with ST-Link V2 clones.
    // Root cause: WFE sleep deactivates the AHB bus matrix, making SWD AHB-AP inaccessible
    // regardless of DBGMCU settings. Use BOOT0 recovery procedure instead.
    // May be effective with CMSIS-DAP probes (e.g. picoprobe) that support true connect-under-reset.
    // config.enable_debug_during_sleep = true;
    let p = embassy_stm32::init(config);
    info!("Embassy STM32F4 Color LED Rainbow started!");

    // PC13: onboard status LED (active low)
    let mut status_led = Output::new(p.PC13, Level::High, Speed::Low);

    // Setup TIM4 PWM for RGB LED
    // PB6 = TIM4_CH1 (Red)
    // PB7 = TIM4_CH2 (Green)
    // PB8 = TIM4_CH3 (Blue)
    let ch1_pin = PwmPin::new(p.PB6, embassy_stm32::gpio::OutputType::PushPull);
    let ch2_pin = PwmPin::new(p.PB7, embassy_stm32::gpio::OutputType::PushPull);
    let ch3_pin = PwmPin::new(p.PB8, embassy_stm32::gpio::OutputType::PushPull);

    let pwm = SimplePwm::new(
        p.TIM4,
        Some(ch1_pin),
        Some(ch2_pin),
        Some(ch3_pin),
        None,
        Hertz::khz(1), // 1kHz PWM frequency
        CountingMode::EdgeAlignedUp,
    );

    // Split PWM into individual channels
    let mut channels = pwm.split();
    let max_duty = channels.ch1.max_duty_cycle();
    info!("PWM max duty cycle: {}", max_duty);

    // Enable PWM channels
    channels.ch1.enable();
    channels.ch2.enable();
    channels.ch3.enable();

    let mut hue: u16 = 0;
    let mut toggle_counter: u8 = 0;

    loop {
        // Convert HSV to RGB (full saturation and brightness)
        let (r, g, b) = hsv_to_rgb(hue, 255, 255);

        // Set PWM duty cycles (scale 0-255 to 0-max_duty)
        let duty_r = r as u32 * max_duty / 255;
        let duty_g = g as u32 * max_duty / 255;
        let duty_b = b as u32 * max_duty / 255;

        channels.ch1.set_duty_cycle(duty_r);
        channels.ch2.set_duty_cycle(duty_g);
        channels.ch3.set_duty_cycle(duty_b);

        // Increment hue for rainbow effect
        hue = (hue + 1) % 360;

        // Toggle status LED every ~1 second (20 * 50ms)
        toggle_counter += 1;
        if toggle_counter >= 20 {
            status_led.toggle();
            toggle_counter = 0;
            info!("Hue: {}, RGB: ({}, {}, {})", hue, r, g, b);
        }

        // Update every 50ms for smooth color transition
        Timer::after_millis(50).await;
    }
}
