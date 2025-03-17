use alloc::format;
use alloc::{boxed::Box, string::String};
use async_trait::async_trait;
use embedded_graphics::{prelude::*, text::Text};
use embedded_layout::prelude::*;
use fugit::HertzU32;
use micromath::F32Ext;

use crate::app::AppMode;
use crate::{
    app::{Draw, Update},
    peripherals::{AnalogInput, PeripheralError},
};

pub struct BuzzerMode<'a> {
    input: Box<dyn AnalogInput + 'a>,
    output: Box<dyn BuzzerOutput + 'a>,
    frequency: Option<HertzU32>,
}

impl<'a> BuzzerMode<'a> {
    const FREQ_MIN: HertzU32 = HertzU32::Hz(10);
    const FREQ_MAX: HertzU32 = HertzU32::Hz(500);

    pub fn new(input: impl AnalogInput + 'a, output: impl BuzzerOutput + 'a) -> Self {
        Self {
            input: Box::new(input),
            output: Box::new(output),
            frequency: None,
        }
    }
}

fn interp_log(pct: f32, min: f32, max: f32) -> f32 {
    let min_log = min.log10();
    let max_log = max.log10();
    let d = max_log - min_log;
    let x = min_log + (pct / 100.0) * d;
    10.0f32.powf(x)
}

#[async_trait]
pub trait BuzzerOutput: Send {
    async fn enable(&mut self) -> Result<(), PeripheralError>;

    async fn disable(&mut self) -> Result<(), PeripheralError>;

    async fn set_frequency(&mut self, freq: HertzU32) -> Result<(), PeripheralError>;
}

#[async_trait]
impl Update for BuzzerMode<'_> {
    async fn update(&mut self) {
        let input = self.input.input_pct().await;
        if let Ok(pct) = input {
            // Set set_frequency
            let freq = interp_log(
                pct,
                Self::FREQ_MIN.to_Hz() as f32,
                Self::FREQ_MAX.to_Hz() as f32,
            );
            let freq = HertzU32::Hz(freq as u32);
            self.frequency = Some(freq);
            _ = self.output.set_frequency(freq).await;
        }
    }
}

impl<D> Draw<D> for BuzzerMode<'_>
where
    D: DrawTarget,
{
    fn draw_with_style(
        &self,
        style: &crate::app::AppStyle<<D as DrawTarget>::Color>,
        draw_area: embedded_graphics::primitives::Rectangle,
        target: &mut D,
    ) -> Result<(), <D as DrawTarget>::Error> {
        let string = match self.frequency {
            Some(freq) => format!("{:.2}Hz", freq.to_Hz()),
            None => String::from("???"),
        };

        let text = Text::new(&string, Point::zero(), style.text_style.clone());

        text.align_to(&draw_area, horizontal::Center, vertical::Center)
            .draw(target)?;

        Ok(())
    }
}

#[async_trait]
impl<D> AppMode<D> for BuzzerMode<'_>
where
    D: DrawTarget,
{
    fn title(&self) -> String {
        String::from("Buzzer")
    }

    async fn enter(&mut self) -> Result<(), PeripheralError> {
        self.output.enable().await?;
        if let Some(freq) = self.frequency {
            self.output.set_frequency(freq).await?;
        }
        Ok(())
    }

    async fn exit(&mut self) -> Result<(), PeripheralError> {
        self.output.disable().await
    }
}
