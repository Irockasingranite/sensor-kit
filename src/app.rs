use embedded_graphics::{prelude::*, primitives::Rectangle};
use u8g2_fonts::U8g2TextStyle;

pub trait Update {
    fn update(&mut self);
}

pub trait Draw<D, C, E>
where
    D: DrawTarget<Color = C, Error = E>,
{
    fn draw_with_style(
        &self,
        style: &AppStyle<C>,
        draw_area: Rectangle,
        target: &mut D,
    ) -> Result<(), E>;
}

pub struct AppStyle<C> {
    pub title_style: U8g2TextStyle<C>,
    pub text_style: U8g2TextStyle<C>,
    pub default_color: C
}

impl<C> AppStyle<C> {
    pub fn new(title_style: U8g2TextStyle<C>, text_style: U8g2TextStyle<C>, default_color: C) -> Self {
        Self {
            title_style,
            text_style,
            default_color,
        }
    }
}
