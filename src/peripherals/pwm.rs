use alloc::boxed::Box;
use async_trait::async_trait;
use fugit::HertzU32;

use crate::mode::{buzzer::BuzzerOutput, led::LedOutput};

use super::PeripheralError;

#[async_trait]
/// A PWM with additional capabilities compared to embedded_hal's `SetDutyCycle`.
pub trait Pwm {
    /// Set duty cycle in percent.
    async fn set_duty_cycle_percent(&mut self, percent: u8) -> Result<(), PeripheralError>;

    /// Set PWM frequency.
    async fn set_frequency(&mut self, freq: HertzU32) -> Result<(), PeripheralError>;

    /// Enable PWM output.
    async fn enable(&mut self) -> Result<(), PeripheralError>;

    /// Disable PWM output.
    async fn disable(&mut self) -> Result<(), PeripheralError>;
}

#[async_trait]
impl<P> LedOutput for P
where
    P: Pwm + Send,
{
    async fn set_brightness(&mut self, brightness_pct: f32) -> Result<(), PeripheralError> {
        self.set_duty_cycle_percent(brightness_pct as u8).await?;
        Ok(())
    }

    async fn enable(&mut self) -> Result<(), PeripheralError> {
        self.set_frequency(HertzU32::Hz(10000)).await?;
        self.enable().await?;
        Ok(())
    }

    async fn disable(&mut self) -> Result<(), PeripheralError> {
        self.disable().await
    }
}

#[async_trait]
impl<P> BuzzerOutput for P
where
    P: Pwm + Send,
{
    async fn enable(&mut self) -> Result<(), PeripheralError> {
        self.set_duty_cycle_percent(50).await?;
        self.enable().await?;
        Ok(())
    }

    async fn disable(&mut self) -> Result<(), PeripheralError> {
        self.disable().await
    }

    async fn set_frequency(&mut self, freq: fugit::HertzU32) -> Result<(), PeripheralError> {
        let hertz = freq.to_Hz();
        let freq = HertzU32::Hz(hertz);
        self.set_frequency(freq).await?;
        Ok(())
    }
}
