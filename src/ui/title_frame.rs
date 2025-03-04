use embedded_graphics::geometry::AnchorY;
use embedded_graphics::primitives::{
    PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Styled,
};
use embedded_graphics::text::renderer::CharacterStyle;
use embedded_graphics::{
    prelude::*,
    text::{renderer::TextRenderer, Text},
};
use embedded_layout::prelude::*;

pub struct TitleFrame<'a, V, S, C>
where
    V: View,
    C: PixelColor,
    S: CharacterStyle<Color = C>,
{
    inner: V,
    outer_area: Rectangle,
    title: Text<'a, S>,
    frame: Styled<Rectangle, PrimitiveStyle<C>>,
}

impl<'a, V, S, C> TitleFrame<'a, V, S, C>
where
    V: View,
    C: PixelColor,
    S: CharacterStyle<Color = C> + TextRenderer<Color = C>,
{
    pub fn new_with_area(
        mut inner: V,
        title: &'a str,
        title_style: S,
        area: Rectangle,
        color: C,
    ) -> Self {
        let title = Text::new(title, Point::zero(), title_style).align_to(
            &area,
            horizontal::Center,
            vertical::Top,
        );
        let title_height = title.bounding_box().size.height;

        let remaining_area = area.resized_height(area.size.height - title_height, AnchorY::Bottom);

        let frame = remaining_area.offset(-2);
        let inner_area = frame.offset(-2);
        let frame_style = PrimitiveStyleBuilder::new()
            .stroke_color(color)
            .stroke_width(1)
            .stroke_alignment(StrokeAlignment::Outside)
            .build();

        let frame_styled = frame.into_styled(frame_style);

        inner.align_to_mut(&inner_area, horizontal::Center, vertical::Center);

        Self {
            inner,
            outer_area: area,
            title,
            frame: frame_styled,
        }
    }
}

impl<V, S, C> Drawable for TitleFrame<'_, V, S, C>
where
    V: View + Drawable<Color = C>,
    C: PixelColor,
    S: CharacterStyle<Color = C> + TextRenderer<Color = C>,
{
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.title.draw(target)?;
        self.frame.draw(target)?;
        self.inner.draw(target)?;
        Ok(())
    }
}
