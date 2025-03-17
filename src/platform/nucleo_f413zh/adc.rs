use crate::peripherals::{AnalogInput, PeripheralError};

use alloc::boxed::Box;
use alloc::sync::Arc;
use async_trait::async_trait;
use embassy_stm32::adc::{self, Adc as HalAdc, AnyAdcChannel};
use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};

/// An analog input measured by an ADC.
pub struct Adc<'a, ADC, M>
where
    ADC: adc::Instance,
    M: RawMutex + Sync + Send,
{
    /// ADC measuring the signal.
    adc: Arc<Mutex<M, HalAdc<'a, ADC>>>,
    /// ADC channel.
    channel: AnyAdcChannel<ADC>,
    /// Maximum signal value, used for computing relative value.
    max_value: u16,
}

impl<'a, ADC, M> Adc<'a, ADC, M>
where
    ADC: adc::Instance,
    M: RawMutex + Sync + Send,
{
    /// Create a new input instance.
    pub fn new(
        adc: Arc<Mutex<M, HalAdc<'a, ADC>>>,
        channel: AnyAdcChannel<ADC>,
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
impl<ADC, M> AnalogInput for Adc<'_, ADC, M>
where
    ADC: adc::Instance + Send + Sync,
    M: RawMutex + Send + Sync,
{
    async fn input_raw(&mut self) -> Result<u16, PeripheralError> {
        let mut adc = self.adc.lock().await;
        Ok(adc.blocking_read(&mut self.channel))
    }

    async fn max_value(&self) -> Result<u16, PeripheralError> {
        Ok(self.max_value)
    }
}
