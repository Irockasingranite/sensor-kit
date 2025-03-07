use embedded_graphics::{
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment},
};
use embedded_layout::prelude::*;

/// A circular frame with an inner filled circle displaying a percentage.
pub struct FilledCircle<C> {
    /// Radius of the inner circle as percentage of the outer circle.
    pub fill_pct: f32,
    /// Color of the element.
    pub color: C,
    /// Position and size of the outer circle.
    pub circle: Circle,
}

impl<C> FilledCircle<C> {
    pub fn new(fill_pct: f32, color: C, circle: Circle) -> Self {
        Self {
            fill_pct,
            color,
            circle,
        }
    }
}

impl<C> View for FilledCircle<C> {
    fn translate_impl(&mut self, by: Point) {
        self.circle.translate_impl(by)
    }

    fn bounds(&self) -> Rectangle {
        self.circle.bounds()
    }
}

impl<C> Drawable for FilledCircle<C>
where
    C: PixelColor,
{
    type Color = C;
    type Output = ();
    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let outer_style = PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(self.color)
            .stroke_alignment(StrokeAlignment::Inside)
            .build();

        let inner_style = PrimitiveStyleBuilder::new().fill_color(self.color).build();

        let outer_diameter = self.circle.diameter;
        let inner_diameter = outer_diameter as f32 * self.fill_pct / 100.0;
        let inner_diameter_clamped = u32::min(inner_diameter as u32, outer_diameter - 6);

        let outer_circle = self.circle.into_styled(outer_style);

        let inner_circle = Circle::new(Point::zero(), inner_diameter_clamped)
            .into_styled(inner_style)
            .align_to(
                &outer_circle.bounding_box(),
                horizontal::Center,
                vertical::Center,
            );

        outer_circle.draw(target)?;
        inner_circle.draw(target)?;

        Ok(())
    }
}
