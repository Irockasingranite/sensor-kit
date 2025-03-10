use alloc::boxed::Box;
use async_trait::async_trait;
use embedded_hal::pwm::SetDutyCycle;

use crate::mode::led::LedOutput;

use super::PeripheralError;

pub struct PwmLed<PWM>
where
    PWM: SetDutyCycle + Send,
{
    pwm_channel: PWM,
}

impl<PWM> PwmLed<PWM>
where
    PWM: SetDutyCycle + Send,
{
    pub fn new(pwm_channel: PWM) -> Self {
        Self { pwm_channel }
    }
}

#[async_trait]
impl<PWM> LedOutput for PwmLed<PWM>
where
    PWM: SetDutyCycle + Send,
{
    async fn set_brightness(&mut self, brightness_pct: f32) -> Result<(), PeripheralError> {
        self.pwm_channel
            .set_duty_cycle_percent(brightness_pct as u8)
            .map_err(|_| PeripheralError::PwmError)?;

        Ok(())
    }
}
