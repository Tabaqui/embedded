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
use embassy_time::Timer;
use panic_probe as _;
use stm32g030::PidCalc;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    let mut adc = Adc::new_with_clock(p.ADC1, Clock::Async { div: Presc::DIV1 });
    let mut adc_pin1 = p.PA2;
    let mut adc_pin2 = p.PA3;

    let mut vrefint = adc.enable_vrefint();
    let vrefint_sample = adc.blocking_read(&mut vrefint, SampleTime::CYCLES160_5);
    let convert_to_millivolts = |sample: u16| {
        // From https://www.st.com/resource/en/datasheet/stm32g031g8.pdf
        // 6.3.3 Embedded internal reference voltage
        const VREFINT_MV: u32 = 1212; // mV

        (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
    };

    let pwm_pin = PwmPin::new(p.PA8, OutputType::PushPull);
    let mut pwm = SimplePwm::new(
        p.TIM1,
        Some(pwm_pin),
        None,
        None,
        None,
        hz(5000),
        Default::default(),
    );

    let mut adc_values: [f32; 20] = [0.0; 20];
    let mut i = 0;

    let mut pid = PidCalc::new();
    let a = pid.duty();
    pwm.ch1().set_duty_cycle(2700);
    pwm.ch1().enable();
    let mut calc_pid = false;
    Timer::after_secs(5).await;

    loop {
        pwm.ch1().disable();

        Timer::after_micros(600).await;
        let v = adc.blocking_read(&mut adc_pin1, SampleTime::CYCLES7_5);
        let v2 = adc.blocking_read(&mut adc_pin2, SampleTime::CYCLES7_5);
        let d: f32 = convert_to_millivolts(v2) as f32 - convert_to_millivolts(v) as f32;
        adc_values[i] = d;
        if i > 18 {
            let avg = adc_values.iter().sum::<f32>() / 20.0;
            i = 0;
            info!("avg {}", avg);
        }
        i += 1;
        // info!("d {}", d);

        pwm.ch1().enable();

        Timer::after_millis(100).await;
    }
}
