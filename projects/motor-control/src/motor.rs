use embassy_stm32::gpio::Output;
use embassy_stm32::peripherals::TIM3;
use embassy_stm32::timer::simple_pwm::{SimplePwm, SimplePwmChannel};

/// Motor rotation direction for TB6612FNG IN1/IN2 logic
pub enum Direction {
    Forward,
    Reverse,
    Stop,  // coast: both pins low
    Brake, // short brake: both pins high
}

/// Motor identifier for DualMotor::set_motor
#[allow(dead_code)]
pub enum MotorId {
    A,
    B,
}

/// Single motor direction controller.
///
/// Manages IN1/IN2 GPIO pins. Speed control (PWM duty) is delegated to DualMotor.
pub struct Motor {
    in1: Output<'static>,
    in2: Output<'static>,
}

impl Motor {
    pub fn new(in1: Output<'static>, in2: Output<'static>) -> Self {
        Self { in1, in2 }
    }

    pub fn set_direction(&mut self, dir: Direction) {
        match dir {
            Direction::Forward => { self.in1.set_high(); self.in2.set_low(); }
            Direction::Reverse => { self.in1.set_low();  self.in2.set_high(); }
            Direction::Stop    => { self.in1.set_low();  self.in2.set_low(); }
            Direction::Brake   => { self.in1.set_high(); self.in2.set_high(); }
        }
    }
}

/// Dual motor controller for differential drive.
///
/// Owns TIM3 CH1/CH2 channels and two Motor instances.
/// Provides both per-motor control and high-level drive commands.
pub struct DualMotor {
    pwm_a: SimplePwmChannel<'static, TIM3>, // CH1 = PB4
    pwm_b: SimplePwmChannel<'static, TIM3>, // CH2 = PB5
    motor_a: Motor,
    motor_b: Motor,
    max_duty: u32,
}

impl DualMotor {
    pub fn new(pwm: SimplePwm<'static, TIM3>, motor_a: Motor, motor_b: Motor) -> Self {
        let mut channels = pwm.split();
        let max_duty = channels.ch1.max_duty_cycle();
        channels.ch1.enable();
        channels.ch2.enable();
        Self {
            pwm_a: channels.ch1,
            pwm_b: channels.ch2,
            motor_a,
            motor_b,
            max_duty,
        }
    }

    fn speed_to_duty(&self, speed: u8) -> u32 {
        speed as u32 * self.max_duty / 255
    }

    /// Set direction and speed (0-255) for a single motor
    #[allow(dead_code)]
    pub fn set_motor(&mut self, id: MotorId, dir: Direction, speed: u8) {
        let duty = self.speed_to_duty(speed);
        match id {
            MotorId::A => {
                self.motor_a.set_direction(dir);
                self.pwm_a.set_duty_cycle(duty);
            }
            MotorId::B => {
                self.motor_b.set_direction(dir);
                self.pwm_b.set_duty_cycle(duty);
            }
        }
    }

    /// Both motors forward at given speed (0-255)
    pub fn forward(&mut self, speed: u8) {
        let duty = self.speed_to_duty(speed);
        self.motor_a.set_direction(Direction::Forward);
        self.motor_b.set_direction(Direction::Forward);
        self.pwm_a.set_duty_cycle(duty);
        self.pwm_b.set_duty_cycle(duty);
    }

    /// Both motors reverse at given speed (0-255)
    pub fn backward(&mut self, speed: u8) {
        let duty = self.speed_to_duty(speed);
        self.motor_a.set_direction(Direction::Reverse);
        self.motor_b.set_direction(Direction::Reverse);
        self.pwm_a.set_duty_cycle(duty);
        self.pwm_b.set_duty_cycle(duty);
    }

    /// Pivot turn left: motor A reverse, motor B forward
    pub fn turn_left(&mut self, speed: u8) {
        let duty = self.speed_to_duty(speed);
        self.motor_a.set_direction(Direction::Reverse);
        self.motor_b.set_direction(Direction::Forward);
        self.pwm_a.set_duty_cycle(duty);
        self.pwm_b.set_duty_cycle(duty);
    }

    /// Pivot turn right: motor A forward, motor B reverse
    pub fn turn_right(&mut self, speed: u8) {
        let duty = self.speed_to_duty(speed);
        self.motor_a.set_direction(Direction::Forward);
        self.motor_b.set_direction(Direction::Reverse);
        self.pwm_a.set_duty_cycle(duty);
        self.pwm_b.set_duty_cycle(duty);
    }

    /// Coast stop: both motors coast to a halt
    pub fn stop(&mut self) {
        self.motor_a.set_direction(Direction::Stop);
        self.motor_b.set_direction(Direction::Stop);
        self.pwm_a.set_duty_cycle(0);
        self.pwm_b.set_duty_cycle(0);
    }

    /// Short brake: both motors braked
    pub fn brake(&mut self) {
        self.motor_a.set_direction(Direction::Brake);
        self.motor_b.set_direction(Direction::Brake);
        self.pwm_a.set_duty_cycle(0);
        self.pwm_b.set_duty_cycle(0);
    }
}
