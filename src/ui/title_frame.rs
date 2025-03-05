use embedded_graphics::geometry::AnchorY;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle, StrokeAlignment};
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::text::Baseline;
use embedded_graphics::{
    prelude::*,
    text::{renderer::TextRenderer, Text},
};
use embedded_layout::prelude::*;

pub struct TitleFrame<'a, S, C> {
    title: &'a str,
    title_style: S,
    frame_color: C,
    area: Rectangle,
}

impl<'a, S, C> TitleFrame<'a, S, C>
where
    S: TextRenderer,
{
    pub fn new(title: &'a str, title_style: S, frame_color: C, area: Rectangle) -> Self {
        Self {
            title,
            title_style,
            frame_color,
            area,
        }
    }

    pub fn inner_area(&self) -> Rectangle {
        self.area_below_title().offset(-5)
    }

    fn area_below_title(&self) -> Rectangle {
        let title_height = self
            .title_style
            .measure_string(self.title, Point::zero(), Baseline::Bottom)
            .bounding_box
            .size
            .height;

        self.area
            .resized_height(self.area.size.height - title_height, AnchorY::Bottom)
    }
}

impl<S, C> View for TitleFrame<'_, S, C>
where
    S: TextRenderer,
{
    fn translate_impl(&mut self, by: Point) {
        self.area.translate_impl(by);
    }

    fn bounds(&self) -> Rectangle {
        self.area.bounds()
    }
}

impl<S, C> Drawable for TitleFrame<'_, S, C>
where
    C: PixelColor,
    S: CharacterStyle<Color = C> + TextRenderer<Color = C>,
{
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let title = Text::new(self.title, Point::zero(), self.title_style.clone()).align_to(
            &self.area,
            horizontal::Center,
            vertical::Top,
        );

        let frame_style = PrimitiveStyleBuilder::new()
            .stroke_color(self.frame_color)
            .stroke_width(1)
            .stroke_alignment(StrokeAlignment::Outside)
            .build();

        let frame = self.area_below_title().offset(-3).into_styled(frame_style);

        title.draw(target)?;
        frame.draw(target)?;

        Ok(())
    }
}
