use alloc::boxed::Box;
use async_trait::async_trait;
use embassy_rp::pwm::{self, Pwm, SetDutyCycle};
use embassy_time::Timer;
use fugit::HertzU32;

use crate::peripherals::{PeripheralError, Pwm as PwmTrait};

pub struct PwmPin<'a> {
    slice: Pwm<'a>,
    config: pwm::Config,
}

impl<'a> PwmPin<'a> {
    pub fn new(mut slice: Pwm<'a>, config: pwm::Config) -> Self {
        slice.set_config(&config);
        Self { slice, config }
    }
}

#[async_trait]
impl<'a> PwmTrait for PwmPin<'a> {
    async fn set_duty_cycle_percent(&mut self, percent: u8) -> Result<(), PeripheralError> {
        self.slice
            .set_duty_cycle_percent(percent)
            .map_err(|_| PeripheralError::Pwm)
    }

    async fn enable(&mut self) -> Result<(), PeripheralError> {
        self.config.enable = true;
        self.slice.set_config(&self.config);
        Ok(())
    }

    async fn disable(&mut self) -> Result<(), PeripheralError> {
        self.slice
            .set_duty_cycle(0)
            .map_err(|_| PeripheralError::Pwm)?;
        Timer::after_millis(10).await; // Ensure the pin actually goes low
        self.config.enable = false;
        self.slice.set_config(&self.config);
        Ok(())
    }

    async fn set_frequency(&mut self, freq: HertzU32) -> Result<(), PeripheralError> {
        let freq_hz = freq.to_Hz();
        let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
        let divider = 255u8; // TODO: Select this depending on frequency range
        let period = (clock_freq_hz / (freq_hz * divider as u32)) as u16 - 1;

        self.config.top = period;
        self.config.divider = divider.into();

        self.slice.set_config(&self.config);
        self.slice
            .set_duty_cycle_percent(50)
            .map_err(|_| PeripheralError::Pwm)?;

        Ok(())
    }
}
