#![no_main]
#![no_std]

use cortex_m::prelude::_embedded_hal_Pwm;
use defmt::*;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, Clock, Presc, SampleTime};
use embassy_stm32::gpio::OutputType;
use embassy_stm32::i2c::Error::Timeout;
use embassy_stm32::time::{hz, Hertz};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::{Duration, Ticker, Timer};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");
    _spawner.spawn(periodic().unwrap());

    let mut ticker = Ticker::every(Duration::from_secs(60));
    loop {
        ticker.next().await;
        info!("60 secs gone...")
    }
}

#[embassy_executor::task]
async fn periodic() {
    let mut tuc_tuc = Ticker::every(Duration::from_secs(1));

    loop {
        tuc_tuc.next().await;

        info!("Tuc");
    }
}
