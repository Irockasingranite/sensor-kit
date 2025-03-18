use alloc::{boxed::Box, format, string::String};
use async_trait::async_trait;
use embedded_graphics::{prelude::*, text::Text};
use embedded_layout::prelude::*;

use crate::{
    app::{AppMode, Draw, Update},
    peripherals::{AnalogInput, PeripheralError},
};

/// Struct defining the 'LED Mode'. Sampled from an analog input and sets LED brightness
/// accordingly.
pub struct LedMode<'a> {
    /// LED output.
    led: Box<dyn LedOutput + 'a>,
    /// Input determining the brightness.
    input: Box<dyn AnalogInput + Send + 'a>,
    /// Current brightness, if any.
    brightness_pct: Option<f32>,
}

impl<'a> LedMode<'a> {
    pub fn new(led: impl LedOutput + 'a, input: impl AnalogInput + 'a) -> Self {
        Self {
            led: Box::new(led),
            input: Box::new(input),
            brightness_pct: None,
        }
    }
}

#[async_trait]
/// An LED output with variable brightness.
pub trait LedOutput: Send {
    /// Set brightness in percent.
    async fn set_brightness(&mut self, brightness_pct: f32) -> Result<(), PeripheralError>;

    /// Enable LED output.
    async fn enable(&mut self) -> Result<(), PeripheralError> {
        Ok(())
    }

    /// Disable LED output.
    async fn disable(&mut self) -> Result<(), PeripheralError> {
        self.set_brightness(0.0).await
    }
}

#[async_trait]
impl Update for LedMode<'_> {
    async fn update(&mut self) {
        self.brightness_pct = self.input.input_pct().await.ok();
        let brightness = self.brightness_pct.unwrap_or(0.0);
        _ = self.led.set_brightness(brightness).await;
    }
}

impl<D> Draw<D> for LedMode<'_>
where
    D: DrawTarget,
{
    fn draw_with_style(
        &self,
        style: &crate::app::AppStyle<<D as DrawTarget>::Color>,
        draw_area: embedded_graphics::primitives::Rectangle,
        target: &mut D,
    ) -> Result<(), <D as DrawTarget>::Error> {
        let string = match self.brightness_pct {
            Some(b) => format!("{b:.2}%"),
            None => String::from("???"),
        };

        let text = Text::new(&string, Point::zero(), style.text_style.clone());

        text.align_to(&draw_area, horizontal::Center, vertical::Center)
            .draw(target)?;

        Ok(())
    }
}

#[async_trait]
impl<D> AppMode<D> for LedMode<'_>
where
    D: DrawTarget,
{
    fn title(&self) -> String {
        String::from("Led")
    }

    async fn enter(&mut self) -> Result<(), PeripheralError> {
        self.led.enable().await
    }

    async fn exit(&mut self) -> Result<(), PeripheralError> {
        self.led.disable().await
    }
}
