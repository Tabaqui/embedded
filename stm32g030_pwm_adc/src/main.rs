#![no_main]
#![no_std]

use cortex_m::asm::nop;
use cortex_m::interrupt::Mutex;
use cortex_m::prelude::_embedded_hal_Pwm;
use defmt::*;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, AdcChannel, AnyAdcChannel, Clock, Presc, SampleTime};
use embassy_stm32::gpio::OutputType;
use embassy_stm32::i2c::Error::Timeout;
use embassy_stm32::interrupt::Interrupt::ADC1;
// use embassy_stm32::interrupt::Interrupt::DMA1_CHANNEL1;
use embassy_stm32::peripherals::{self, PA2, PA3, TIM1};
use embassy_stm32::time::{hz, Hertz};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Dma;
use embassy_stm32::{bind_interrupts, dma, Peri};
use embassy_time::{Duration, Ticker, Timer};
use panic_probe as _;
use stm32g030::PidCalc;
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;


static SHARED_CHANNEL: Channel<ThreadModeRawMutex, f32, 8> = Channel::new();

// static ADCC:

static mut DMA_BUF: [u16; 20] = [0; 20];

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // let mut read_buffer = unsafe { &mut DMA_BUF[..] };

    let adc = Adc::new_with_clock(p.ADC1, Clock::Async { div: Presc::DIV1 });
    let adc_pin1 = p.PA2.degrade_adc();
    let adc_pin2 = p.PA3.degrade_adc();



    let pwm_pin = PwmPin::new(p.PA8, OutputType::PushPull);
    let pwm = SimplePwm::new(
        p.TIM1,
        Some(pwm_pin),
        None,
        None,
        None,
        hz(5000),
        Default::default(),
    );

    // // --

    // let mut adc_values: [f32; 20] = [0.0; 20];
    // let mut i = 0;

    // let mut pid = PidCalc::new();
    // let a = pid.duty();
    // pwm.ch1().set_duty_cycle(2700);
    // pwm.ch1().enable();
    // let mut calc_pid = false;
    // Timer::after_secs(5).await;


    let dma: Peri<'_, peripherals::DMA1_CH1> = p.DMA1_CH1;

    _spawner.spawn(calc_adc_diff(dma, pwm, adc, adc_pin1, adc_pin2).unwrap());

    loop {
        let filtred_diff_mv = SHARED_CHANNEL.receiver().receive().await;

        info!("Received {}", filtred_diff_mv);
        


        // Timer::after_millis(100).await;

        // let mut dma: Peri<'_, peripherals::DMA1_CH1> = p.DMA1_CH1;
    }
}

#[embassy_executor::task]
async fn calc_adc_diff(
    mut dma: Peri<'static, peripherals::DMA1_CH1>,
    mut pwm: SimplePwm<'static, TIM1>,
    mut adc: Adc<'static, peripherals::ADC1>,
    mut adc_pin1: AnyAdcChannel<'static, peripherals::ADC1>,
    mut adc_pin2: AnyAdcChannel<'static, peripherals::ADC1>,
) {
    let mut adc_ticker = Ticker::every(Duration::from_millis(100));

    let mut r_buffer = unsafe { &mut DMA_BUF[..] };

    let mut filtred_diff_mv = 0.0;

    let mut vrefint = adc.enable_vrefint();
    let vrefint_sample = adc.blocking_read(&mut vrefint, SampleTime::CYCLES160_5);
    let to_mv = |sample: u16| {
        // From https://www.st.com/resource/en/datasheet/stm32g031g8.pdf
        // 6.3.3 Embedded internal reference voltage
        const VREFINT_MV: u32 = 1212;
        (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
    };

    loop {
        adc_ticker.next().await;
        pwm.ch1().disable();
        Timer::after_micros(500).await;

        let adc_channels = [
            (&mut adc_pin1, SampleTime::CYCLES160_5),
            (&mut adc_pin2, SampleTime::CYCLES160_5),
        ]
        .into_iter();

        adc.read(dma.reborrow(), Irqs, adc_channels, &mut r_buffer)
            .await;

        let adc_v_ref_1 = r_buffer[0];
        let adc_v_ref_2 = r_buffer[1];

        let diff_mv = to_mv(adc_v_ref_1) as f32 - to_mv(adc_v_ref_2) as f32;

        filtred_diff_mv = (1.0 - 0.2) * filtred_diff_mv + 0.2 * diff_mv;
        if let Err(e) = SHARED_CHANNEL.sender().try_send(filtred_diff_mv) {
            error!("Channel is {}", e);
        };
        // info!("filtred_diff_mv: {}", filtred_diff_mv);
        pwm.ch1().enable();
    }
}
