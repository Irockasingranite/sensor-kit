use alloc::{boxed::Box, string::String};
use async_trait::async_trait;
use embedded_graphics::{prelude::*, primitives::Circle};
use embedded_layout::prelude::*;

use crate::app::{AppMode, Draw, Update};
use crate::peripherals::AnalogInput;
use crate::ui::FilledCircle;

/// Struct defining the 'Light Sensor' mode. Samples data from a photo-resistor and displays value
/// as a partially filled circle.
pub struct LightSensorMode<'a> {
    input: Box<dyn AnalogInput + 'a>,
    value_pct: Option<f32>,
}

impl<'a> LightSensorMode<'a> {
    pub fn new(input: impl AnalogInput + 'a) -> Self {
        Self {
            input: Box::new(input),
            value_pct: None,
        }
    }
}

#[async_trait]
impl Update for LightSensorMode<'_> {
    async fn update(&mut self) {
        self.value_pct = self.input.input_pct().await.ok();
    }
}

impl<D> Draw<D> for LightSensorMode<'_>
where
    D: DrawTarget,
{
    fn draw_with_style(
        &self,
        style: &crate::app::AppStyle<<D as embedded_graphics::prelude::DrawTarget>::Color>,
        draw_area: embedded_graphics::primitives::Rectangle,
        target: &mut D,
    ) -> Result<(), <D as embedded_graphics::prelude::DrawTarget>::Error> {
        let max_dimension = u32::min(draw_area.size.width, draw_area.size.height);
        let diameter = max_dimension as f32 * 0.9;

        let circle = FilledCircle::new(
            self.value_pct.unwrap_or(0.0),
            style.default_color,
            Circle::new(Point::zero(), diameter as u32),
        )
        .align_to(&draw_area, horizontal::Center, vertical::Center);

        circle.draw(target)?;

        Ok(())
    }
}

#[async_trait]
impl<D> AppMode<D> for LightSensorMode<'_>
where
    D: DrawTarget,
{
    fn title(&self) -> alloc::string::String {
        String::from("Light Sensor")
    }
}
