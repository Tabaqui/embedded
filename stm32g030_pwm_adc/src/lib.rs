#![no_std]

const P: f64 = 0.5;

const TARGET_VOLTS: f32 = 2500.0;

pub struct PidCalc {
    volts: f32,
    duty: u16,
}

impl PidCalc {
    pub fn new() -> Self {
        PidCalc {
            volts: 0.0,
            duty: 200,
        }
    }

    pub fn update_rpm(&mut self, back_volts: f32) -> i32 {
        // let income = new_value - self.value;
        self.volts = back_volts;
        self.update_duty()
    }

    fn update_duty(&mut self) -> i32 {
        let mis = TARGET_VOLTS - self.volts;

        if self.duty > 1000 || mis > -20.0 && mis < 20.0 {
            return 0;
        }

        return if mis > 0.0 {
            self.duty += 1;
            1
        } else {
            self.duty -= 1;
            -1
        };
    }

    pub fn duty(&self) -> u16 {
        self.duty
    }
}
