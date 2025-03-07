use alloc::boxed::Box;
use alloc::sync::Arc;
use async_trait::async_trait;
use embassy_stm32::adc::{self, Adc, AdcChannel};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use crate::mode::potentiometer::PotentiometerInput;

use super::PeripheralError;

/// Struct containing peripherals used for sampling potentiometer setting.
pub struct SensorKitPotentiometer<'a, ADC, CH>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
{
    /// ADC measuring the signal.
    adc: Arc<Mutex<CriticalSectionRawMutex, Adc<'a, ADC>>>,
    /// Specific ADC channel used.
    channel: CH,
}

impl<'a, ADC, CH> SensorKitPotentiometer<'a, ADC, CH>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
{
    const MAX_VALUE: u16 = 4096;

    pub fn new(adc: Arc<Mutex<CriticalSectionRawMutex, Adc<'a, ADC>>>, channel: CH) -> Self {
        Self { adc, channel }
    }
}

#[async_trait]
impl<ADC, CH> PotentiometerInput for SensorKitPotentiometer<'_, ADC, CH>
where
    ADC: adc::Instance + Send,
    CH: adc::AdcChannel<ADC> + Send,
{
    async fn value_pct(&mut self) -> Result<f32, PeripheralError> {
        let mut adc = self.adc.lock().await;
        let value = adc.blocking_read(&mut self.channel);
        let inverted = Self::MAX_VALUE - value;
        let value_pct = 100.0 * (inverted as f32 / Self::MAX_VALUE as f32);
        Ok(value_pct)
    }
}
