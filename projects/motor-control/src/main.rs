//! Motor Control demo using Embassy on STM32F411CE (Black Pill)
//!
//! GPIO Configuration:
//!   - PB4 = TIM3_CH1 (PWM-A, Motor A speed)
//!   - PB5 = TIM3_CH2 (PWM-B, Motor B speed)
//!   - PB6 = AIN2 (Motor A direction)
//!   - PB7 = AIN1 (Motor A direction)
//!   - PB8 = BIN1 (Motor B direction)
//!   - PB9 = BIN2 (Motor B direction)
//!   - PC13 = Status LED (active low)

#![no_std]
#![no_main]

mod motor;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::Timer;
use motor::{DualMotor, Motor};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = embassy_stm32::Config::default();
    let p = embassy_stm32::init(config);
    info!("Embassy STM32F4 Motor Control started!");

    // PC13: onboard status LED (active low)
    let mut status_led = Output::new(p.PC13, Level::Low, Speed::Low);

    // TIM3 CH1/CH2: PWM for motor speed control (20kHz, above audible range)
    let pwm_a_pin = PwmPin::new(p.PB4, embassy_stm32::gpio::OutputType::PushPull);
    let pwm_b_pin = PwmPin::new(p.PB5, embassy_stm32::gpio::OutputType::PushPull);
    let pwm = SimplePwm::new(
        p.TIM3,
        Some(pwm_a_pin),
        Some(pwm_b_pin),
        None,
        None,
        Hertz::khz(20),
        CountingMode::EdgeAlignedUp,
    );

    // Motor A direction: AIN1=PB7, AIN2=PB6
    let motor_a = Motor::new(
        Output::new(p.PB7, Level::Low, Speed::High), // AIN1
        Output::new(p.PB6, Level::Low, Speed::High), // AIN2
    );

    // Motor B direction: BIN1=PB8, BIN2=PB9
    let motor_b = Motor::new(
        Output::new(p.PB8, Level::Low, Speed::High), // BIN1
        Output::new(p.PB9, Level::Low, Speed::High), // BIN2
    );

    let mut drive = DualMotor::new(pwm, motor_a, motor_b);
    info!("Motor driver initialized");

    loop {
        info!("Forward 50%");
        status_led.set_low(); // active low: LED on
        drive.forward(128);
        Timer::after_millis(2000).await;

        info!("Stop (coast)");
        drive.stop();
        Timer::after_millis(1000).await;

        info!("Backward 50%");
        drive.backward(128);
        Timer::after_millis(2000).await;

        info!("Stop (coast)");
        drive.stop();
        Timer::after_millis(1000).await;

        info!("Turn left 50%");
        drive.turn_left(128);
        Timer::after_millis(1000).await;

        info!("Turn right 50%");
        drive.turn_right(128);
        Timer::after_millis(1000).await;

        info!("Brake");
        drive.brake();
        status_led.set_high(); // LED off
        Timer::after_millis(1000).await;
    }
}
