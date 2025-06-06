#![no_std]
#![no_main]

use cortex_m_rt::entry;

use panic_halt as _;
// use stm32f1xx_hal::pac as Pa;
use stm32f4xx_hal::{
    gpio::Pin,
    pac::{self},
    prelude::*,
};


#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let gpioa = dp.GPIOA.split();
    let gpioc = dp.GPIOC.split();   

    let mut led = gpioa.pa5.into_push_pull_output();

    let btn = gpioc.pc13;

    let mut del_var = 10_0000u32;

    led.set_low();

    loop {
        del_var = loop_delay(del_var, &btn);

        led.toggle();
    }
}

fn loop_delay<const P: char, const N: u8>(mut del: u32, but: &Pin<P, N>) -> u32 {
 for _i in 1..del {
        if but.is_low() {
            del = del - 2_5000_u32;
            if del < 2_5000 {
                del = 10_0000_u32;
            }
            return del;
        }
    }
    del
}