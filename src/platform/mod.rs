#[cfg(feature = "nucleo-f413zh")]
pub mod nucleo_f413zh;
#[cfg(feature = "rp-pico")]
pub mod rp_pico;

use crate::peripherals::AnalogInput;
use crate::peripherals::Pwm;

use alloc::boxed::Box;
use async_trait::async_trait;
use embedded_hal::i2c::I2c;

pub struct Platform<I2C, AIN, PWM, PIN>
where
    I2C: I2c,
    AIN: AnalogInput,
    PWM: Pwm,
    PIN: DynSafeWait,
{
    pub i2c: I2C,
    pub a0: AIN,
    pub a2: AIN,
    pub a3: AIN,
    pub d4: PIN,
    pub d5: PWM,
    pub d6: PWM,
}

impl<I2C, AIN, PWM, PIN> Platform<I2C, AIN, PWM, PIN>
where
    I2C: I2c,
    AIN: AnalogInput,
    PWM: Pwm,
    PIN: DynSafeWait,
{
    pub fn new(i2c: I2C, a0: AIN, a2: AIN, a3: AIN, d4: PIN, d5: PWM, d6: PWM) -> Self {
        Self {
            i2c,
            a0,
            a2,
            a3,
            d4,
            d5,
            d6,
        }
    }

    pub fn split(self) -> (I2C, AIN, AIN, AIN, PIN, PWM, PWM) {
        (
            self.i2c, self.a0, self.a2, self.a3, self.d4, self.d5, self.d6,
        )
    }
}

#[allow(dead_code)]
#[async_trait]
pub trait DynSafeWait {
    type Error;
    async fn wait_for_high(&mut self) -> Result<(), Self::Error>;
    async fn wait_for_low(&mut self) -> Result<(), Self::Error>;
    async fn wait_for_rising_edge(&mut self) -> Result<(), Self::Error>;
    async fn wait_for_falling_edge(&mut self) -> Result<(), Self::Error>;
    async fn wait_for_any_edge(&mut self) -> Result<(), Self::Error>;
}
