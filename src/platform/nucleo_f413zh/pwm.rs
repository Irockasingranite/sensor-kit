use crate::peripherals::{PeripheralError, Pwm};

use alloc::boxed::Box;
use alloc::sync::Arc;
use async_trait::async_trait;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::{simple_pwm::SimplePwm, Channel, GeneralInstance4Channel};
use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};

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

    async fn set_frequency(&mut self, freq: fugit::HertzU32) -> Result<(), PeripheralError> {
        let mut pwm = self.pwm.lock().await;
        let freq = Hertz::hz(freq.to_Hz());
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
