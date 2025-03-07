use embassy_stm32::adc::{self, Adc, AdcChannel};

use crate::mode::potentiometer::PotentiometerInput;

use super::PeripheralError;

/// Struct containing peripherals used for sampling potentiometer setting.
pub struct SensorKitPotentiometer<'a, ADC, CH>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
{
    /// ADC measuring the signal.
    adc: Adc<'a, ADC>,
    /// Specific ADC channel used.
    channel: CH,
}

impl<'a, ADC, CH> SensorKitPotentiometer<'a, ADC, CH>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
{
    const MAX_VALUE: u16 = 4096;

    pub fn new(adc: Adc<'a, ADC>, channel: CH) -> Self {
        Self { adc, channel }
    }
}

impl<ADC, CH> PotentiometerInput for SensorKitPotentiometer<'_, ADC, CH>
where
    ADC: adc::Instance,
    CH: adc::AdcChannel<ADC>,
{
    fn value_pct(&mut self) -> Result<f32, PeripheralError> {
        let value = self.adc.blocking_read(&mut self.channel);
        let inverted = Self::MAX_VALUE - value;
        let value_pct = 100.0 * (inverted as f32 / Self::MAX_VALUE as f32);
        Ok(value_pct)
    }
}
