use accelerometer::vector::F32x3;
use alloc::{boxed::Box, string::String};
use async_trait::async_trait;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyleBuilder};

use crate::app::AppMode;
use crate::{
    app::{Draw, Update},
    peripherals::PeripheralError,
};

pub struct AccelerationMode<'a> {
    input: Box<dyn AccelerationInput + 'a>,
    acceleration: Option<F32x3>,
}

impl<'a> AccelerationMode<'a> {
    pub fn new(input: impl AccelerationInput + 'a) -> Self {
        Self {
            input: Box::new(input),
            acceleration: None,
        }
    }
}

#[async_trait]
pub trait AccelerationInput: Send {
    async fn accel_norm(&mut self) -> Result<F32x3, PeripheralError>;
}

#[async_trait]
impl Update for AccelerationMode<'_> {
    async fn update(&mut self) {
        self.acceleration = self.input.accel_norm().await.ok();
    }
}

impl<D> Draw<D> for AccelerationMode<'_>
where
    D: DrawTarget,
{
    fn draw_with_style(
        &self,
        style: &crate::app::AppStyle<<D as embedded_graphics::prelude::DrawTarget>::Color>,
        draw_area: embedded_graphics::primitives::Rectangle,
        target: &mut D,
    ) -> Result<(), <D as embedded_graphics::prelude::DrawTarget>::Error> {
        if self.acceleration.is_none() {
            return Ok(());
        }

        let line_origin = draw_area.center();
        let smallest_dimension = u32::min(draw_area.size.width, draw_area.size.height);
        let max_length = (smallest_dimension as f32 * 0.9) / 2.0;

        // Flipped due to physical sensor orientation relative to display
        let x = -self.acceleration.unwrap().y * max_length;
        let y = -self.acceleration.unwrap().x * max_length;

        let line_end = Point::new(x as i32, y as i32) + line_origin;

        let line = Line::new(line_origin, line_end);
        let line_style = PrimitiveStyleBuilder::new()
            .stroke_color(style.default_color)
            .stroke_width(1)
            .build();

        line.into_styled(line_style).draw(target)?;

        Ok(())
    }
}

#[async_trait]
impl<D> AppMode<D> for AccelerationMode<'_>
where
    D: DrawTarget,
{
    fn title(&self) -> String {
        String::from("Acceleration")
    }
}
