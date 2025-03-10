use alloc::{boxed::Box, string::String};
use async_trait::async_trait;
use embedded_graphics::{prelude::*, primitives::Rectangle};
use embedded_layout::prelude::*;

use super::inputs::PctInput;
use crate::app::{AppMode, Draw, Update};
use crate::ui::HorizontalBar;

// Struct defining the 'Sound Sensor' mode. Sample data from the sound sensor and displays it as a
// bar.
pub struct SoundMode<'a> {
    /// Input used.
    input: Box<dyn PctInput + 'a>,
    /// Value in %.
    value_pct: Option<f32>,
}

impl<'a> SoundMode<'a> {
    pub fn new(input: impl PctInput + 'a) -> Self {
        Self {
            input: Box::new(input),
            value_pct: None,
        }
    }
}

#[async_trait]
impl Update for SoundMode<'_> {
    async fn update(&mut self) {
        self.value_pct = self.input.input_pct().await.ok();
    }
}

impl<D> Draw<D> for SoundMode<'_>
where
    D: DrawTarget,
{
    fn draw_with_style(
        &self,
        style: &crate::app::AppStyle<<D as DrawTarget>::Color>,
        draw_area: Rectangle,
        target: &mut D,
    ) -> Result<(), <D as DrawTarget>::Error> {
        let bar_width = draw_area.size.width as f32 * 0.9;
        let bar_height = draw_area.size.height as f32 * 0.9;

        let bar = HorizontalBar::new(
            self.value_pct.unwrap_or(0.0),
            style.default_color,
            Rectangle::new(
                Point::zero(),
                Size::new(bar_width as u32, bar_height as u32),
            ),
        )
        .align_to(&draw_area, horizontal::Center, vertical::Center);

        bar.draw(target)?;

        Ok(())
    }
}

#[async_trait]
impl<D> AppMode<D> for SoundMode<'_>
where
    D: DrawTarget,
{
    fn title(&self) -> alloc::string::String {
        String::from("Sound Sensor")
    }
}
