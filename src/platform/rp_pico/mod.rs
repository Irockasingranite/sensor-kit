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

mod adc {
    use alloc::boxed::Box;
    use alloc::sync::Arc;
    use async_trait::async_trait;
    use embassy_rp::adc::{self, Adc as HalAdc, Async};
    use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};

    use crate::peripherals::{AnalogInput, PeripheralError};

    pub struct Adc<'a, M>
    where
        M: RawMutex,
    {
        adc: Arc<Mutex<M, HalAdc<'a, Async>>>,
        channel: adc::Channel<'a>,
        max_value: u16,
    }

    impl<'a, M> Adc<'a, M>
    where
        M: RawMutex,
    {
        pub fn new(
            adc: Arc<Mutex<M, HalAdc<'a, Async>>>,
            channel: adc::Channel<'a>,
            max_value: Option<u16>,
        ) -> Self {
            Self {
                adc,
                channel,
                max_value: max_value.unwrap_or(4096),
            }
        }
    }

    #[async_trait]
    impl<M> AnalogInput for Adc<'_, M>
    where
        M: RawMutex + Send + Sync,
    {
        async fn input_raw(&mut self) -> Result<u16, PeripheralError> {
            let mut adc = self.adc.lock().await;
            adc.blocking_read(&mut self.channel)
                .map_err(|_| PeripheralError::Adc)
        }

        async fn max_value(&self) -> Result<u16, PeripheralError> {
            Ok(self.max_value)
        }
    }
}

mod pwm {
    use alloc::boxed::Box;
    use async_trait::async_trait;
    use embassy_rp::pwm::{self, Pwm, SetDutyCycle};
    use fugit::HertzU32;
    use embassy_time::Timer;

    use crate::peripherals::{PeripheralError, Pwm as PwmTrait};

    pub struct PwmPin<'a> {
        slice: Pwm<'a>,
        config: pwm::Config,
    }

    impl<'a> PwmPin<'a> {
        pub fn new(mut slice: Pwm<'a>, config: pwm::Config) -> Self {
            slice.set_config(&config);
            Self { slice, config }
        }
    }

    #[async_trait]
    impl<'a> PwmTrait for PwmPin<'a> {
        async fn set_duty_cycle_percent(&mut self, percent: u8) -> Result<(), PeripheralError> {
            self.slice
                .set_duty_cycle_percent(percent)
                .map_err(|_| PeripheralError::Pwm)
        }

        async fn enable(&mut self) -> Result<(), PeripheralError> {
            self.config.enable = true;
            self.slice.set_config(&self.config);
            Ok(())
        }

        async fn disable(&mut self) -> Result<(), PeripheralError> {
            self.slice
                .set_duty_cycle(0)
                .map_err(|_| PeripheralError::Pwm)?;
            Timer::after_millis(10).await; // Ensure the pin actually goes low
            self.config.enable = false;
            self.slice.set_config(&self.config);
            Ok(())
        }

        async fn set_frequency(&mut self, freq: HertzU32) -> Result<(), PeripheralError> {
            let freq_hz = freq.to_Hz();
            let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
            let divider = 255u8; // TODO: Select this depending on frequency range
            let period = (clock_freq_hz / (freq_hz * divider as u32)) as u16 - 1;

            self.config.top = period;
            self.config.divider = divider.into();

            self.slice.set_config(&self.config);
            self.slice
                .set_duty_cycle_percent(50)
                .map_err(|_| PeripheralError::Pwm)?;

            Ok(())
        }
    }
}
