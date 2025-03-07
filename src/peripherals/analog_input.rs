use alloc::boxed::Box;
use alloc::sync::Arc;
use async_trait::async_trait;
use embassy_stm32::adc::{self, Adc, AdcChannel};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use crate::mode::light::LightSensor;
use crate::mode::potentiometer::PotentiometerInput;
use crate::mode::sound::SoundInput;

use super::PeripheralError;

/// An analog input measured by an ADC.
pub struct AnalogInput<'a, ADC, CH>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
{
    /// ADC measuring the signal.
    adc: Arc<Mutex<CriticalSectionRawMutex, Adc<'a, ADC>>>,
    /// ADC channel.
    channel: CH,
    /// Maximum signal value, used for computing relative value.
    max_value: u16,
}

impl<'a, ADC, CH> AnalogInput<'a, ADC, CH>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
{
    /// Create a new input instance.
    pub fn new(
        adc: Arc<Mutex<CriticalSectionRawMutex, Adc<'a, ADC>>>,
        channel: CH,
        max_value: u16,
    ) -> Self {
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

#[async_trait]
impl<ADC, CH> PotentiometerInput for AnalogInput<'_, ADC, CH>
where
    ADC: adc::Instance + Send,
    CH: AdcChannel<ADC> + Send,
{
    async fn get_value(&mut self) -> Result<f32, super::PeripheralError> {
        // Potentiometer input is reversed
        let input = self.get_value_pct().await?;
        let reversed = 100.0 - input;
        Ok(reversed)
    }
}

#[async_trait]
impl<ADC, CH> LightSensor for AnalogInput<'_, ADC, CH>
where
    ADC: adc::Instance + Send,
    CH: AdcChannel<ADC> + Send,
{
    async fn get_value(&mut self) -> Result<f32, super::PeripheralError> {
        self.get_value_pct().await
    }
}

#[async_trait]
impl<ADC, CH> SoundInput for AnalogInput<'_, ADC, CH>
where
    ADC: adc::Instance + Send,
    CH: AdcChannel<ADC> + Send,
{
    async fn get_value(&mut self) -> Result<f32, super::PeripheralError> {
        self.get_value_pct().await
    }
}
