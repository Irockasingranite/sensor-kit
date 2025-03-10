use alloc::{boxed::Box, sync::Arc};
use async_trait::async_trait;
use embassy_stm32::{
    time::Hertz,
    timer::{simple_pwm::SimplePwm, Channel, GeneralInstance4Channel},
};
use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};

use crate::mode::{buzzer::BuzzerOutput, led::LedOutput};

use super::PeripheralError;

#[async_trait]
trait Pwm {
    async fn set_duty_cycle_percent(&mut self, percent: u8) -> Result<(), PeripheralError>;

    async fn set_frequency(&mut self, freq: Hertz) -> Result<(), PeripheralError>;

    async fn enable(&mut self) -> Result<(), PeripheralError>;

    async fn disable(&mut self) -> Result<(), PeripheralError>;
}

pub struct SharedPwm<'a, M, I>
where
    M: RawMutex,
    I: GeneralInstance4Channel,
{
    pwm: Arc<Mutex<M, SimplePwm<'a, I>>>,
    channel: Channel,
}

impl<'a, M, I> SharedPwm<'a, M, I>
where
    M: RawMutex,
    I: GeneralInstance4Channel,
{
    pub fn new(pwm: Arc<Mutex<M, SimplePwm<'a, I>>>, channel: Channel) -> Self {
        Self { pwm, channel }
    }
}

#[async_trait]
impl<M, I> Pwm for SharedPwm<'_, M, I>
where
    M: RawMutex + Send + Sync,
    I: GeneralInstance4Channel + Send,
{
    async fn set_duty_cycle_percent(&mut self, percent: u8) -> Result<(), PeripheralError> {
        let mut pwm = self.pwm.lock().await;
        pwm.channel(self.channel).set_duty_cycle_percent(percent);
        Ok(())
    }

    async fn set_frequency(&mut self, freq: Hertz) -> Result<(), PeripheralError> {
        let mut pwm = self.pwm.lock().await;
        pwm.set_frequency(freq);
        Ok(())
    }

    async fn enable(&mut self) -> Result<(), PeripheralError> {
        let mut pwm = self.pwm.lock().await;
        pwm.channel(self.channel).enable();
        Ok(())
    }

    async fn disable(&mut self) -> Result<(), PeripheralError> {
        let mut pwm = self.pwm.lock().await;
        pwm.channel(self.channel).disable();
        Ok(())
    }
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
        self.set_frequency(Hertz::hz(100)).await?;
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
        let freq = Hertz::hz(hertz);
        self.set_frequency(freq).await?;
        Ok(())
    }
}
