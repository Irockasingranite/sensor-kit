use alloc::boxed::Box;
use alloc::sync::Arc;
use async_trait::async_trait;
use embassy_stm32::adc::{self, Adc, AdcChannel};
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::mutex::Mutex;

use super::PeripheralError;
use crate::mode::inputs::PctInput;

/// An analog input measured by an ADC.
pub struct AnalogInput<'a, ADC, CH, M>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
    M: RawMutex + Sync + Send,
{
    /// ADC measuring the signal.
    adc: Arc<Mutex<M, Adc<'a, ADC>>>,
    /// ADC channel.
    channel: CH,
    /// Maximum signal value, used for computing relative value.
    max_value: u16,
}

impl<'a, ADC, CH, M> AnalogInput<'a, ADC, CH, M>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
    M: RawMutex + Sync + Send,
{
    /// Create a new input instance.
    pub fn new(adc: Arc<Mutex<M, Adc<'a, ADC>>>, channel: CH, max_value: u16) -> Self {
        Self {
            adc,
            channel,
            max_value,
        }
    }

    /// Get the raw value returned by the ADC.
    pub async fn get_value_raw(&mut self) -> Result<u16, PeripheralError> {
        let mut adc = self.adc.lock().await;
        let value = adc.blocking_read(&mut self.channel);
        Ok(value)
    }

    /// Get the signal value relative to the defined maximum.
    pub async fn get_value_pct(&mut self) -> Result<f32, PeripheralError> {
        let value_raw = self.get_value_raw().await?;
        let pct = 100.0 * (value_raw as f32 / self.max_value as f32);
        Ok(pct)
    }
}

pub struct ReversedAnalogInput<'a, ADC, CH, M>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
    M: RawMutex + Send + Sync,
{
    inner: AnalogInput<'a, ADC, CH, M>,
}

impl<'a, ADC, CH, M> ReversedAnalogInput<'a, ADC, CH, M>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
    M: RawMutex + Send + Sync,
{
    pub fn new(adc: Arc<Mutex<M, Adc<'a, ADC>>>, channel: CH, max_value: u16) -> Self {
        Self {
            inner: AnalogInput::new(adc, channel, max_value),
        }
    }
}

#[async_trait]
impl<ADC, CH, M> PctInput for AnalogInput<'_, ADC, CH, M>
where
    ADC: adc::Instance + Send,
    CH: AdcChannel<ADC> + Send,
    M: RawMutex + Send + Sync,
{
    async fn input_pct(&mut self) -> Result<f32, PeripheralError> {
        let value_raw = self.get_value_raw().await?;
        let pct = 100.0 * (value_raw as f32 / self.max_value as f32);
        Ok(pct)
    }
}

#[async_trait]
impl<ADC, CH, M> PctInput for ReversedAnalogInput<'_, ADC, CH, M>
where
    ADC: adc::Instance + Send,
    CH: AdcChannel<ADC> + Send,
    M: RawMutex + Send + Sync,
{
    async fn input_pct(&mut self) -> Result<f32, PeripheralError> {
        let value = self.inner.get_value_pct().await?;
        let reversed = 100.0 - value;
        Ok(reversed)
    }
}
