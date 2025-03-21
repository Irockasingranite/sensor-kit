use alloc::boxed::Box;
use alloc::string::String;
use async_trait::async_trait;
use embedded_graphics::{prelude::*, primitives::Rectangle};
use u8g2_fonts::U8g2TextStyle;

use crate::peripherals::PeripheralError;

/// Marks an object that can update its internal state.
#[async_trait]
pub trait Update {
    /// Update internal state. Using the `#[async_trait]` macro, this function's signature is simply
    /// `async fn update(&mut self)`.
    async fn update(&mut self);
}

/// Marks an object that can be drawn to a target as part of an application.
pub trait Draw<D>
where
    D: DrawTarget,
{
    /// Draw the object into a specific area following the provided style.
    fn draw_with_style(
        &self,
        style: &AppStyle<D::Color>,
        draw_area: Rectangle,
        target: &mut D,
    ) -> Result<(), D::Error>;
}

/// Styling information for application elements.
pub struct AppStyle<C> {
    /// Style used for titles.
    pub title_style: U8g2TextStyle<C>,
    /// Style used for general text elements.
    pub text_style: U8g2TextStyle<C>,
    /// Default color for elements.
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

#[async_trait]
/// An Application mode that can perform an update->draw loop.
pub trait AppMode<D>: Update + Draw<D> + Send
where
    D: DrawTarget,
{
    /// Returns the title that should be displayed.
    fn title(&self) -> String;

    async fn enter(&mut self) -> Result<(), PeripheralError> {
        Ok(())
    }

    async fn exit(&mut self) -> Result<(), PeripheralError> {
        Ok(())
    }
}
