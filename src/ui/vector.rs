use embedded_graphics::{
    prelude::*,
    primitives::{Circle, Line, PrimitiveStyle, Rectangle, StyledDrawable},
};

pub struct Vector {
    origin: Point,
    vector: Point,
}

impl Vector {
    pub fn new(origin: Point, vector: Point) -> Self {
        Self { origin, vector }
    }
}

impl Dimensions for Vector {
    fn bounding_box(&self) -> Rectangle {
        let abs = self.vector.abs();
        let max_dim = i32::max(abs.x, abs.y);
        let circle = Circle::with_center(self.origin, 2 * max_dim as u32);
        circle.bounding_box()
    }
}

impl Primitive for Vector {}

impl<C> StyledDrawable<PrimitiveStyle<C>> for Vector
where
    C: PixelColor,
{
    type Color = C;
    type Output = ();

    fn draw_styled<D>(
        &self,
        style: &PrimitiveStyle<C>,
        target: &mut D,
    ) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let end_point = self.origin + self.vector;
        let line = Line::new(self.origin, end_point);
        line.draw_styled(style, target)
    }
}
