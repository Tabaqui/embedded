#![no_std]
#![no_main]

use cortex_m as _;
use defmt_rtt as _;


use cortex_m_rt::entry;
// use cortex_m::asm;
use panic_halt as _;
use stm32f4xx_hal::{gpio::GpioExt, pac as Pa};


#[entry]
fn main() -> ! {
    let d = Pa::Peripherals::take().unwrap();
    let a = d.GPIOC.split();
    let mut l = a.pc13.into_push_pull_output();
    // let val = 3;
    loop {
        // if 6 < val {
            // let v = d.GPIOA.split();
            // if 6 > val {
                l.toggle();
            // };
        // }
    }
}