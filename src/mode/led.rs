use alloc::{boxed::Box, format, string::String};
use async_trait::async_trait;
use embedded_graphics::{prelude::*, text::Text};
use embedded_layout::prelude::*;

use crate::{
    app::{AppMode, Draw, Update},
    peripherals::PeripheralError,
};

pub struct LedMode<'a> {
    led: Box<dyn LedOutput + 'a>,
    input: Box<dyn LedModeInput + 'a>,
    brightness_pct: Option<f32>,
}

impl<'a> LedMode<'a> {
    pub fn new(led: impl LedOutput + 'a, input: impl LedModeInput + 'a) -> Self {
        Self {
            led: Box::new(led),
            input: Box::new(input),
            brightness_pct: None,
        }
    }
}

#[async_trait]
pub trait LedOutput: Send {
    async fn set_brightness(&mut self, brightness_pct: f32) -> Result<(), PeripheralError>;
}

#[async_trait]
pub trait LedModeInput: Send {
    async fn get_value(&mut self) -> Result<f32, PeripheralError>;
}

#[async_trait]
impl Update for LedMode<'_> {
    async fn update(&mut self) {
        self.brightness_pct = self.input.get_value().await.ok();
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

impl<D> AppMode<D> for LedMode<'_>
where
    D: DrawTarget,
{
    fn title(&self) -> String {
        String::from("Led")
    }
}
