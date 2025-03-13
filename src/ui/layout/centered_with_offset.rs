use embedded_graphics::{prelude::*, primitives::Rectangle};
use embedded_layout::prelude::*;

pub struct CenteredWithOffset<V, W> {
    outer: V,
    inner: W,
    offset: Point,
}

impl<V, W> CenteredWithOffset<V, W> {
    pub fn new(outer: V, inner: W, offset: Point) -> Self {
        Self {
            outer,
            inner,
            offset,
        }
    }
}

impl<V, W> CenteredWithOffset<V, W>
where
    V: View,
    W: View,
{
    pub fn arrange(mut self) -> Self {
        self.inner
            .align_to_mut(&self.outer.bounds(), horizontal::Center, vertical::Center)
            .translate_mut(self.offset);
        self
    }
}

impl<V, W, C, O> Drawable for CenteredWithOffset<V, W>
where
    V: Drawable<Color = C, Output = O>,
    W: Drawable<Color = C, Output = O>,
    C: PixelColor,
{
    type Color = C;
    type Output = O;

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.outer.draw(target)?;
        self.inner.draw(target)
    }
}

impl<V, W> View for CenteredWithOffset<V, W>
where
    V: View,
    W: View,
{
    fn translate_impl(&mut self, by: Point) {
        self.outer.translate_impl(by);
        self.inner.translate_impl(by);
    }

    fn bounds(&self) -> Rectangle {
        let outer_bounds = self.outer.bounds();
        let inner_bounds = self.inner.bounds();
        outer_bounds.enveloping(&inner_bounds)
    }
}
