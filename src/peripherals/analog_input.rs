use alloc::boxed::Box;
use alloc::sync::Arc;
use async_trait::async_trait;
use embassy_stm32::adc::{self, Adc, AdcChannel};
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::mutex::Mutex;

use crate::mode::led::LedModeInput;
use crate::mode::light::LightSensor;
use crate::mode::potentiometer::PotentiometerInput;
use crate::mode::sound::SoundInput;

use super::PeripheralError;

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
trait ValuePct {
    async fn get_value_pct(&mut self) -> Result<f32, PeripheralError>;
}

#[async_trait]
impl<ADC, CH, M> ValuePct for AnalogInput<'_, ADC, CH, M>
where
    ADC: adc::Instance + Send,
    CH: AdcChannel<ADC> + Send,
    M: RawMutex + Send + Sync,
{
    async fn get_value_pct(&mut self) -> Result<f32, PeripheralError> {
        let value_raw = self.get_value_raw().await?;
        let pct = 100.0 * (value_raw as f32 / self.max_value as f32);
        Ok(pct)
    }
}

#[async_trait]
impl<ADC, CH, M> ValuePct for ReversedAnalogInput<'_, ADC, CH, M>
where
    ADC: adc::Instance + Send,
    CH: AdcChannel<ADC> + Send,
    M: RawMutex + Send + Sync,
{
    async fn get_value_pct(&mut self) -> Result<f32, PeripheralError> {
        let value = self.inner.get_value_pct().await?;
        let reversed = 100.0 - value;
        Ok(reversed)
    }
}

#[async_trait]
impl<V> PotentiometerInput for V
where
    V: ValuePct + Send,
{
    async fn get_value(&mut self) -> Result<f32, PeripheralError> {
        self.get_value_pct().await
    }
}

#[async_trait]
impl<V, M> PotentiometerInput for Arc<Mutex<M, V>>
where
    V: ValuePct + Send,
    M: RawMutex + Send + Sync,
{
    async fn get_value(&mut self) -> Result<f32, PeripheralError> {
        let mut input = self.lock().await;
        PotentiometerInput::get_value(&mut *input).await
    }
}

#[async_trait]
impl<V> LightSensor for V
where
    V: ValuePct + Send,
{
    async fn get_value(&mut self) -> Result<f32, PeripheralError> {
        self.get_value_pct().await
    }
}

#[async_trait]
impl<V> SoundInput for V
where
    V: ValuePct + Send,
{
    async fn get_value(&mut self) -> Result<f32, PeripheralError> {
        self.get_value_pct().await
    }
}

#[async_trait]
impl<V> LedModeInput for V
where
    V: ValuePct + Send,
{
    async fn get_value(&mut self) -> Result<f32, PeripheralError> {
        self.get_value_pct().await
    }
}

#[async_trait]
impl<V, M> LedModeInput for Arc<Mutex<M, V>>
where
    V: ValuePct + Send,
    M: RawMutex + Send + Sync,
{
    async fn get_value(&mut self) -> Result<f32, PeripheralError> {
        let mut input = self.lock().await;
        LedModeInput::get_value(&mut *input).await
    }
}
