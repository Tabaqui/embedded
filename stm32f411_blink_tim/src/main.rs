#![no_std]
#![no_main]

use cortex_m::{self as _, asm::nop};
// use defmt::info;
use defmt_rtt as _;


use cortex_m_rt::entry;
// use cortex_m::asm;
use panic_halt as _;
use stm32f4xx_hal::{hal::pwm::SetDutyCycle, pac::{self}, prelude::*};



#[entry]
fn main() -> ! {
    let hp = pac::Peripherals::take().unwrap();

    let rcc = hp.RCC.constrain();

    let clocks = rcc.cfgr.use_hse(1.kHz()).freeze();

    let (_, (pwm_ch1, ..)) = hp.TIM1.pwm_us(100.micros(), &clocks);
    // let (pwm_manager, (pwm_ch1, ..)) = hp.TIM1.pwm_hz(100.kHz(), &clocks);

    
    let gpioa = hp.GPIOA.split();

    // let s = gpioa.pa8.in o_alternate();
    let mut c1 = pwm_ch1.with(gpioa.pa8);
    c1.set_duty_cycle_percent(10).unwrap();

    c1.enable();

    loop {
         nop();
    }
}