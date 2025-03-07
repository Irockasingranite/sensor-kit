use alloc::boxed::Box;
use alloc::sync::Arc;
use async_trait::async_trait;
use embassy_stm32::adc::{self, Adc, AdcChannel};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use crate::mode::light::LightSensor;

pub struct SensorKitLightSensor<'a, ADC, CH>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
{
    adc: Arc<Mutex<CriticalSectionRawMutex, Adc<'a, ADC>>>,
    channel: CH,
}

impl<'a, ADC, CH> SensorKitLightSensor<'a, ADC, CH>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
{
    const MAX_VALUE: u16 = 2800;

    pub fn new(adc: Arc<Mutex<CriticalSectionRawMutex, Adc<'a, ADC>>>, channel: CH) -> Self {
        Self { adc, channel }
    }
}

#[async_trait]
impl<ADC, CH> LightSensor for SensorKitLightSensor<'_, ADC, CH>
where
    ADC: adc::Instance + Send,
    CH: AdcChannel<ADC> + Send,
{
    async fn get_value(&mut self) -> Result<f32, super::PeripheralError> {
        let mut adc = self.adc.lock().await;
        let value = adc.blocking_read(&mut self.channel);
        let value_pct = 100.0 * (value as f32 / Self::MAX_VALUE as f32);
        Ok(value_pct)
    }
}
