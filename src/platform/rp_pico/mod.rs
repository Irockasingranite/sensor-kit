mod adc;
mod pwm;

use super::{DynSafeWait, Platform};

use adc::Adc;
use alloc::boxed::Box;
use alloc::sync::Arc;
use async_trait::async_trait;
use embassy_rp::{
    adc::{self as hal_adc, Adc as HalAdc},
    bind_interrupts,
    gpio::{self, Input},
    i2c::{self, I2c as HalI2c},
    peripherals::{self, I2C0},
    pwm::Pwm,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embedded_hal_async::digital::Wait;
use pwm::PwmPin;

pub type I2c<'a> = HalI2c<'a, I2C0, i2c::Async>;
pub type PinError = core::convert::Infallible;

bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<peripherals::I2C0>;
    ADC_IRQ_FIFO => hal_adc::InterruptHandler;
});

pub fn platform<'a>() -> Platform<I2c<'a>, Adc<'a, CriticalSectionRawMutex>, PwmPin<'a>, Input<'a>>
{
    let p = embassy_rp::init(Default::default());

    let mut i2c_config: i2c::Config = Default::default();
    i2c_config.frequency = 400000;
    let i2c0 = I2c::new_async(p.I2C0, p.PIN_1, p.PIN_0, Irqs, i2c_config);

    let adc = HalAdc::new(p.ADC, Irqs, Default::default());
    let adc: Arc<Mutex<CriticalSectionRawMutex, _>> = Arc::new(Mutex::new(adc));

    let a0_channel = hal_adc::Channel::new_pin(p.PIN_26, gpio::Pull::Up);

    let a2_channel = hal_adc::Channel::new_pin(p.PIN_27, gpio::Pull::Down);
    let a3_channel = hal_adc::Channel::new_pin(p.PIN_28, gpio::Pull::Down);

    let a0 = Adc::new(adc.clone(), a0_channel, None);
    let a2 = Adc::new(adc.clone(), a2_channel, Some(2800));
    let a3 = Adc::new(adc.clone(), a3_channel, Some(1500));

    let pwm_slice1 = Pwm::new_output_b(p.PWM_SLICE1, p.PIN_3, Default::default());
    let d5 = PwmPin::new(pwm_slice1, Default::default());

    let pwm_slice2 = Pwm::new_output_a(p.PWM_SLICE2, p.PIN_4, Default::default());
    let d6 = PwmPin::new(pwm_slice2, Default::default());

    let d4 = Input::new(p.PIN_2, gpio::Pull::Down);

    Platform::new(i2c0, a0, a2, a3, d4, d5, d6)
}

#[async_trait]
impl DynSafeWait for Input<'_> {
    type Error = core::convert::Infallible;

    async fn wait_for_high(&mut self) -> Result<(), Self::Error> {
        Wait::wait_for_high(&mut self).await
    }

    async fn wait_for_low(&mut self) -> Result<(), Self::Error> {
        Wait::wait_for_low(&mut self).await
    }

    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error> {
        Wait::wait_for_rising_edge(&mut self).await
    }

    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error> {
        Wait::wait_for_falling_edge(&mut self).await
    }

    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error> {
        Wait::wait_for_any_edge(&mut self).await
    }
}
