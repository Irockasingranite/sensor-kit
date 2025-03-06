use embassy_stm32::adc::{self, Adc, AdcChannel};

use crate::mode::potentiometer::PotentiometerInput;

pub struct SensorKitPotentiometer<'a, ADC, CH>
where
    ADC: adc::Instance,
    CH: AdcChannel<ADC>,
{
    adc: Adc<'a, ADC>,
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
    type Error = core::convert::Infallible;

    fn value_pct(&mut self) -> Result<f32, Self::Error> {
        let value = self.adc.blocking_read(&mut self.channel);
        let inverted = Self::MAX_VALUE - value;
        let value_pct = 100.0 * (inverted as f32 / Self::MAX_VALUE as f32);
        Ok(value_pct)
    }
}
