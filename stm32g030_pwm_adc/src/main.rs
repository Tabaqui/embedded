#![no_main]
#![no_std]

use core::cell::RefCell;

use cortex_m::asm::nop;
use cortex_m::prelude::_embedded_hal_Pwm;
use defmt::*;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::yield_now;
use embassy_stm32::adc::{Adc, AdcChannel, AnyAdcChannel, Clock, Presc, SampleTime};
use embassy_stm32::gpio::OutputType;
use embassy_stm32::i2c::Error::Timeout;
use embassy_stm32::interrupt::Interrupt::ADC1;
// use embassy_stm32::interrupt::Interrupt::DMA1_CHANNEL1;
use embassy_stm32::peripherals::{self, PA2, PA3, TIM1};
use embassy_stm32::time::{hz, Hertz};
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::{Ch1, Dma};
use embassy_stm32::{bind_interrupts, dma, Peri};
// use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::{NoopRawMutex, RawMutex, ThreadModeRawMutex};
use embassy_sync::channel::Channel;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Ticker, Timer};
use panic_probe as _;
use static_cell::StaticCell;
use stm32g030::MotorDriver;

static SHARED_CHANNEL: Channel<ThreadModeRawMutex, VBox, 8> = Channel::new();

// static PWM: embassy_sync::mutex::Mutex<ThreadModeRawMutex, RefCell<Option<SimplePwm<'_, TIM1>>>> = Mutex::new(RefCell::new(None));
// static ADCC:

static mut DMA_BUF: [u16; 20] = [0; 20];

type Tf = Mutex<ThreadModeRawMutex, SimplePwm<'static, TIM1>>;
type MoT = Mutex<ThreadModeRawMutex, Mo<'static>>;

// static f:  StaticCell<Tf> = St;
// static SOME_PWM: StaticCell<RefCell<SimplePwm<'static, TIM1>>> = StaticCell::new();

bind_interrupts!(struct Irqs {
    DMA1_CHANNEL1 => dma::InterruptHandler<peripherals::DMA1_CH1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    // let mut read_buffer = unsafe { &mut DMA_BUF[..] };
    // let a = p.PA8
    let mut adc = Adc::new_with_clock(p.ADC1, Clock::Async { div: Presc::DIV1 });
    let adc_pin1 = p.PA2.degrade_adc();
    let adc_pin2 = p.PA3.degrade_adc();

    let pwm_pin: PwmPin<'_, _, Ch1> = PwmPin::new(p.PA8, OutputType::PushPull);

    let mut pwm = SimplePwm::new(
        p.TIM1,
        Some(pwm_pin),
        None,
        None,
        None,
        hz(5000),
        Default::default(),
    );
    // let ppp: Tf = Mutex::new(RefCell::new(pwm));
    // static PWM_CELL: StaticCell<Tf> = StaticCell::new();
    // let ppp = PWM_CELL.init(Mutex::new(pwm));

    let dma: Peri<'_, peripherals::DMA1_CH1> = p.DMA1_CH1;

    static MO_CELL: StaticCell<MoT> = StaticCell::new();
    let ggg = MO_CELL.init(Mutex::new(Mo::new(pwm, adc, adc_pin1, adc_pin2, dma)));

    // pwm.ch1().enable();
    // let mut calc_pid = false;
    // Timer::after_secs(5).await;

    _spawner.spawn(calc_adc_diff(ggg).unwrap());

    // ppp.borrow_mut();
    loop {
        let vbox = SHARED_CHANNEL.receiver().try_receive();
            // info!("ddl");

        if let Err(e) =  vbox {
            // Timer::after_millis(100).await;
            // info!("dl");
            yield_now().await;
            continue;
        }

        let vbox = vbox.unwrap();

        info!("Received {}", vbox.mv());

    //     let mv = 1000.0;

    //     if vbox.current_mv < mv {
    //         info!("lock");
    //         while let Err(l) = ggg.try_lock() {
    //             // info!("try");
    //         };

    //         let mut l = ggg.lock().await;
    //         info!("accured!");
    //         let current_duty = l.pwm.ch1().current_duty_cycle();
    //         drop(l);

    //         let miss = mv - vbox.current_mv;
    //         let set_mv = 0.5 * miss + vbox.current_mv;

    //         info!("cd s_mv {} {}",current_duty,set_mv);
    //     }
    }
}

#[embassy_executor::task]
async fn mesure_ten_secs() {
    // let adc_measure = adc.blocking_read(&mut vrefint, SampleTime::CYCLES160_5);
}

#[embassy_executor::task]
async fn calc_adc_diff(ggg: &'static MoT) {
    let mut adc_ticker = Ticker::every(Duration::from_millis(100));

    let mut r_buffer = unsafe { &mut DMA_BUF[..] };

    let mut filtred_diff_mv = 0.0;

    let mut l = ggg.lock().await;
    let mut vrefint = l.adc.enable_vrefint();
    let vrefint_sample = l.adc.blocking_read(&mut vrefint, SampleTime::CYCLES160_5);
    drop(l);
    let to_mv = |sample: u16| {
        // From https://www.st.com/resource/en/datasheet/stm32g031g8.pdf
        // 6.3.3 Embedded internal reference voltage
        const VREFINT_MV: u32 = 1212;
        (u32::from(sample) * VREFINT_MV / u32::from(vrefint_sample)) as u16
    };

    loop {
        adc_ticker.next().await;

        let mut l = ggg.lock().await;
        l.pwm.ch1().disable();
        drop(l);

        Timer::after_micros(500).await;

        let mut l = ggg.lock().await;
        let Mo {
            adc_pin1,
            adc_pin2,
            adc,
            dma,
            ..
        } = &mut *l;

        // let  a1 = &mut (*l).adc_pin1;
        // let  a2 = &mut (*l).adc_pin2;
        let adc_channels = [
            (adc_pin1, SampleTime::CYCLES160_5),
            (adc_pin2, SampleTime::CYCLES160_5),
        ]
        .into_iter();

        adc.read(dma.reborrow(), Irqs, adc_channels, &mut r_buffer)
            .await;
        drop(l);

        let adc_v_ref_1 = r_buffer[0];
        let adc_v_ref_2 = r_buffer[1];

        info!("Raw: {}, {}", adc_v_ref_1, adc_v_ref_2);

        let diff_mv = to_mv(adc_v_ref_1) as f32 - to_mv(adc_v_ref_2) as f32;

        filtred_diff_mv = (1.0 - 0.2) * filtred_diff_mv + 0.2 * diff_mv;

        let mut l = ggg.lock().await;
        let curr_duty = l.pwm.ch1().current_duty_cycle();
        drop(l);

        if let Err(e) = SHARED_CHANNEL
            .sender()
            .try_send(VBox::new(filtred_diff_mv, curr_duty))
        {
            error!("Channel {}", e);
        };
        // info!("filtred_diff_mv: {}", filtred_diff_mv);

        let mut l = ggg.lock().await;
        l.pwm.ch1().enable();
        drop(l);
    }
}

#[derive(Debug, Format)]
struct VBox {
    current_mv: f32,
    current_duty: u16,
}

impl VBox {
    fn new(mv: f32, duty: u16) -> Self {
        VBox {
            current_mv: mv,
            current_duty: duty,
        }
    }

    pub fn mv(&self) -> f32 {
        self.current_mv
    }

    pub fn duty(&self) -> u16 {
        self.current_duty
    }
}

struct Mo<'a> {
    pwm: SimplePwm<'a, TIM1>,
    adc: Adc<'a, peripherals::ADC1>,
    adc_pin1: AnyAdcChannel<'a, peripherals::ADC1>,
    adc_pin2: AnyAdcChannel<'a, peripherals::ADC1>,
    dma: Peri<'a, peripherals::DMA1_CH1>,
}

impl<'a> Mo<'a> {
    fn new(
        pwm: SimplePwm<'a, TIM1>,
        adc: Adc<'a, peripherals::ADC1>,
        adc_pin1: AnyAdcChannel<'a, peripherals::ADC1>,
        adc_pin2: AnyAdcChannel<'a, peripherals::ADC1>,
        dma: Peri<'a, peripherals::DMA1_CH1>,
    ) -> Self {
        Mo {
            pwm: pwm,
            adc: adc,
            adc_pin1: adc_pin1,
            adc_pin2: adc_pin2,
            dma: dma,
        }
    }
}
