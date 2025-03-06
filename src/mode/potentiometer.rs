use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use embedded_graphics::text::Text;
use embedded_graphics::{prelude::*, primitives::Rectangle};
use embedded_layout::align::Align;
use embedded_layout::layout::linear::{FixedMargin, LinearLayout};
use embedded_layout::prelude::*;

use crate::app::{AppMode, AppStyle};
use crate::peripherals::PeripheralError;
use crate::{
    app::{Draw, Update},
    ui::HorizontalBar,
};

pub struct PotentiometerMode<'a> {
    input: Box<dyn PotentiometerInput + 'a>,
    value_pct: Option<f32>,
}

impl<'a> PotentiometerMode<'a> {
    pub fn new(input: impl PotentiometerInput + 'a) -> Self {
        Self {
            input: Box::new(input),
            value_pct: None,
        }
    }
}

impl Update for PotentiometerMode<'_> {
    fn update(&mut self) {
        self.value_pct = self.input.value_pct().ok();
    }
}

impl<D> Draw<D> for PotentiometerMode<'_>
where
    D: DrawTarget,
{
    fn draw_with_style(
        &self,
        style: &AppStyle<D::Color>,
        draw_area: Rectangle,
        target: &mut D,
    ) -> Result<(), D::Error> {
        let string = match self.value_pct {
            Some(value) => format!("{:.1}%", value),
            None => String::from("???"),
        };

        let text = Text::new(&string, Point::zero(), style.text_style.clone());
        let text_height = text.bounding_box().size.height;

        let bar_width = (0.8 * draw_area.size.width as f32) as u32;
        let bar_height = u32::min(20, draw_area.size.height - text_height);

        let bar_value = self.value_pct.unwrap_or(0.0);
        let bar = HorizontalBar::new(
            bar_value,
            style.default_color,
            Rectangle::new(Point::zero(), Size::new(bar_width, bar_height)),
        );

        LinearLayout::vertical(Chain::new(text).append(bar))
            .with_alignment(horizontal::Center)
            .with_spacing(FixedMargin(5))
            .arrange()
            .align_to(&draw_area, horizontal::Center, vertical::Center)
            .draw(target)?;

        Ok(())
    }
}

impl<D> AppMode<D> for PotentiometerMode<'_>
where
    D: DrawTarget,
{
    fn title(&self) -> String {
        String::from("Potentiometer")
    }
}

pub trait PotentiometerInput {
    fn value_pct(&mut self) -> Result<f32, PeripheralError>;
}
