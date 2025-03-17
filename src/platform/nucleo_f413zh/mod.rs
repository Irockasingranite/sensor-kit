mod adc;
mod pwm;

use super::{DynSafeWait, Platform};
use adc::Adc;
use pwm::SharedPwm;

use alloc::boxed::Box;
use alloc::sync::Arc;
use async_trait::async_trait;
use embassy_stm32::{
    adc::{Adc as HalAdc, AdcChannel},
    bind_interrupts,
    exti::ExtiInput,
    gpio,
    i2c::{self, I2c as HalI2c},
    mode::Async,
    peripherals::{ADC1, TIM1},
    time::Hertz,
    timer::{
        self,
        low_level::CountingMode,
        simple_pwm::{PwmPin, SimplePwm},
    },
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embedded_hal_async::digital::Wait;

bind_interrupts!(struct Irqs {
    I2C1_ER => i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;
    I2C1_EV => i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>;
});

pub type I2c<'a> = HalI2c<'a, Async>;
pub type PinError = core::convert::Infallible;

pub fn platform<'a>() -> Platform<
    HalI2c<'a, Async>,
    Adc<'a, ADC1, CriticalSectionRawMutex>,
    SharedPwm<'a, CriticalSectionRawMutex, TIM1>,
    ExtiInput<'a>,
> {
    let p = embassy_stm32::init(Default::default());

    let i2c1 = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        p.DMA1_CH1,
        p.DMA1_CH0,
        Hertz::khz(400),
        Default::default(),
    );

    let adc = HalAdc::new(p.ADC1);
    let adc: Arc<Mutex<CriticalSectionRawMutex, _>> = Arc::new(Mutex::new(adc));

    let a0_adc_channel = p.PA3.degrade_adc();
    let a2_adc_channel = p.PC3.degrade_adc();
    let a3_adc_channel = p.PC1.degrade_adc();

    let a0 = Adc::new(adc.clone(), a0_adc_channel, None);

    let a2 = Adc::new(adc.clone(), a2_adc_channel, Some(2800));

    let a3 = Adc::new(adc.clone(), a3_adc_channel, Some(1500));

    let d4 = ExtiInput::new(p.PF14, p.EXTI14, gpio::Pull::Down);

    let d5 = PwmPin::new_ch2(p.PE11, gpio::OutputType::PushPull);
    let d6 = PwmPin::new_ch1(p.PE9, gpio::OutputType::PushPull);

    let pwm = SimplePwm::new(
        p.TIM1,
        Some(d6),
        Some(d5),
        None,
        None,
        Hertz::hz(50),
        CountingMode::EdgeAlignedUp,
    );
    let pwm: Arc<Mutex<CriticalSectionRawMutex, _>> = Arc::new(Mutex::new(pwm));

    let d5 = SharedPwm::new(pwm.clone(), timer::Channel::Ch2);
    let d6 = SharedPwm::new(pwm.clone(), timer::Channel::Ch1);

    Platform::new(i2c1, a0, a2, a3, d4, d5, d6)
}

#[async_trait]
impl<'a> DynSafeWait for ExtiInput<'a> {
    type Error = <ExtiInput<'a> as embedded_hal::digital::ErrorType>::Error;

    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        Wait::wait_for_high(self).await
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        Wait::wait_for_low(self).await
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        Wait::wait_for_rising_edge(self).await
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        Wait::wait_for_falling_edge(self).await
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        Wait::wait_for_any_edge(self).await
    }
}
