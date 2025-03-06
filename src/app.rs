use alloc::string::String;
use embedded_graphics::{prelude::*, primitives::Rectangle};
use u8g2_fonts::U8g2TextStyle;

pub trait Update {
    fn update(&mut self);
}

pub trait Draw<D>
where
    D: DrawTarget,
{
    fn draw_with_style(
        &self,
        style: &AppStyle<D::Color>,
        draw_area: Rectangle,
        target: &mut D,
    ) -> Result<(), D::Error>;
}

pub struct AppStyle<C> {
    pub title_style: U8g2TextStyle<C>,
    pub text_style: U8g2TextStyle<C>,
    pub default_color: C,
}

impl<C> AppStyle<C> {
    pub fn new(
        title_style: U8g2TextStyle<C>,
        text_style: U8g2TextStyle<C>,
        default_color: C,
    ) -> Self {
        Self {
            title_style,
            text_style,
            default_color,
        }
    }
}

pub trait AppMode<D>: Update + Draw<D>
where
    D: DrawTarget,
{
    fn title(&self) -> String;
}
