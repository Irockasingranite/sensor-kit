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
