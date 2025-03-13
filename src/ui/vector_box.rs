use core::ops::Mul;
use embedded_graphics::{
    prelude::*,
    primitives::{Line, PrimitiveStyle, Rectangle, Styled, StyledDimensions, StyledDrawable},
};
use embedded_layout::prelude::*;
use micromath::vector::F32x2;

pub struct VectorBox {
    area: Rectangle,
    vector: F32x2,
    vector_scale: f32,
}

impl VectorBox {
    pub fn new(area: Rectangle, vector: F32x2, vector_scale: f32) -> Self {
        Self {
            area,
            vector,
            vector_scale,
        }
    }
}

impl Dimensions for VectorBox {
    fn bounding_box(&self) -> Rectangle {
        self.area.bounding_box()
    }
}

impl Transform for VectorBox {
    fn translate(&self, by: Point) -> Self {
        Self {
            area: self.area.translate(by),
            ..*self
        }
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        Transform::translate_mut(&mut self.area, by);
        self
    }
}

impl Primitive for VectorBox {
    fn into_styled<S>(self, style: S) -> Styled<Self, S>
    where
        Self: Sized,
    {
        Styled {
            primitive: self,
            style,
        }
    }
}

impl<C> StyledDimensions<PrimitiveStyle<C>> for VectorBox
where
    C: PixelColor,
{
    fn styled_bounding_box(&self, style: &PrimitiveStyle<C>) -> Rectangle {
        self.area.styled_bounding_box(style)
    }
}

impl<C> StyledDrawable<PrimitiveStyle<C>> for VectorBox
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
        let frame = self.area;
        let smallest_dimension = u32::min(frame.size.width, frame.size.height);

        let max_line_length = smallest_dimension as f32 * 0.45;
        let line_origin = frame.center();
        let scaled_vec = self.vector.mul(self.vector_scale).mul(max_line_length);
        let line_end = line_origin + Point::new(scaled_vec.x as i32, scaled_vec.y as i32);

        let line = Line::new(line_origin, line_end);

        frame.draw_styled(style, target)?;
        line.draw_styled(style, target)?;

        Ok(())
    }
}
