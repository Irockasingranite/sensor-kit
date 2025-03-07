use embedded_graphics::{
    geometry::AnchorX,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, StrokeAlignment},
};
use embedded_layout::prelude::*;

/// A horizontal, partially filled "loading bar"-like element.
pub struct HorizontalBar<C> {
    /// Percentage of the bar that is filled.
    pub fill_pct: f32,
    /// Color of the bar.
    pub color: C,
    /// Area filled by the bar.
    pub area: Rectangle,
}

impl<C> HorizontalBar<C> {
    pub fn new(fill_pct: f32, color: C, area: Rectangle) -> Self {
        Self {
            fill_pct,
            color,
            area,
        }
    }
}

impl<C> View for HorizontalBar<C> {
    fn translate_impl(&mut self, by: Point) {
        self.area.translate_impl(by)
    }

    fn bounds(&self) -> Rectangle {
        self.area
    }
}

impl<C> Drawable for HorizontalBar<C>
where
    C: PixelColor,
{
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let frame_style = PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(self.color)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();

        let fill_style = PrimitiveStyleBuilder::new().fill_color(self.color).build();

        let frame = self.area.into_styled(frame_style);

        let inner_area = self.area.offset(-2);
        let full_width = inner_area.size.width;
        let fill_width = (full_width as f32 * self.fill_pct / 100.0) as u32;

        let fill = inner_area
            .resized_width(fill_width, AnchorX::Left)
            .into_styled(fill_style);

        frame.draw(target)?;
        if self.fill_pct > 0.0 {
            fill.draw(target)?;
        }

        Ok(())
    }
}
