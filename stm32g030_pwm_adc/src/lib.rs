#![no_std]

use cortex_m::prelude::_embedded_hal_Pwm;
use embassy_stm32::{
    peripherals::TIM1,
    timer::{
        simple_pwm::{PwmPin, SimplePwm},
        Ch1,
    },
    Peri,
};

const P: u32 = 50;

const TARGET_VOLTS: f32 = 2700.0;
const MAX_DUTY: u16 = 1000;
// const THEN_MAX_

pub struct MotorDriver<'a> {
    mv: f32,
    // duty: u16,
    pwm: SimplePwm<'a, TIM1>,
}

impl<'a> MotorDriver<'a> {
    fn new(pwm: SimplePwm<'a, TIM1>) -> Self {
        MotorDriver {
            mv: 0.0,
            // duty: 0,
            pwm: pwm,
        }
    }
    fn start(&mut self) -> Self {
        self.pwm.ch1().set_duty_cycle_percent(20);
        todo!("measure then set mv");
    }

    fn _mv(&self, to_mv: f32) {
        let to_duty = 500;
        todo!("to_mv to to_duty");

        let miss = to_duty - self.pwm.ch1().current_duty_cycle() as u32;

        let proportional = P * miss / 100; // проценты

        self.pwm.ch1().set_duty_cycle(proportional);
    }
}

enum DutyError {
    MaxMVToHigh,
}
