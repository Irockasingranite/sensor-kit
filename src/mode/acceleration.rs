use accelerometer::vector::F32x3;
use alloc::{boxed::Box, string::String};
use async_trait::async_trait;
use core::ops::Mul;
use embassy_time::Instant;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment};
use embedded_layout::align::Align;
use embedded_layout::layout::linear::{FixedMargin, LinearLayout};
use embedded_layout::prelude::*;
use micromath::{vector::F32x2, F32Ext};

use crate::app::AppMode;
use crate::ui::layout::CenteredWithOffset;
use crate::ui::Vector;
use crate::{
    app::{Draw, Update},
    peripherals::PeripheralError,
};

struct Area {
    pub x_max: f32,
    pub x_min: f32,
    pub y_max: f32,
    pub y_min: f32,
}

#[allow(dead_code)]
impl Area {
    pub fn width(&self) -> f32 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> f32 {
        self.y_max - self.y_min
    }
}

pub struct AccelerationMode<'a> {
    input: Box<dyn AccelerationInput + 'a>,
    acceleration: Option<F32x3>,
    ball_position: F32x2,
    ball_velocity: F32x2,
    last_update: Instant,
}

impl<'a> AccelerationMode<'a> {
    const BALL_DIAMETER: u32 = 5;
    const AREA: Area = Area {
        x_max: 1.0,
        x_min: -1.0,
        y_max: 1.0,
        y_min: -1.0,
    };

    pub fn new(input: impl AccelerationInput + 'a) -> Self {
        Self {
            input: Box::new(input),
            acceleration: None,
            ball_position: F32x2 { x: 0.0, y: 0.0 },
            ball_velocity: F32x2 { x: 0.0, y: 0.0 },
            last_update: Instant::now(),
        }
    }
}

#[async_trait]
pub trait AccelerationInput: Send {
    async fn accel_norm(&mut self) -> Result<F32x3, PeripheralError>;
}

#[async_trait]
impl Update for AccelerationMode<'_> {
    async fn update(&mut self) {
        self.acceleration = self.input.accel_norm().await.ok();

        let now = Instant::now();
        if let Some(acc) = self.acceleration {
            // Get time delta in seconds
            let delta = now - self.last_update;
            let delta = (delta.as_millis() as f32) / 200.0;

            // Flipped due to physical orientation of display and sensor
            let mut acc = F32x2 {
                x: -acc.y,
                y: -acc.x,
            };

            // Apply friction
            use micromath::vector::Vector;
            let vel_mag = self.ball_velocity.magnitude();
            let friction = self.ball_velocity.mul(vel_mag.powi(2)).mul(0.01);
            acc -= friction;

            // Update state
            self.ball_velocity += acc.mul(delta);
            self.ball_position += self.ball_velocity.mul(delta);
            self.last_update = now;

            // Collision

            if self.ball_position.x > Self::AREA.x_max {
                self.ball_position.x = Self::AREA.x_max;
                self.ball_velocity.x = 0.0;
            }

            if self.ball_position.x < Self::AREA.x_min {
                self.ball_position.x = Self::AREA.x_min;
                self.ball_velocity.x = 0.0;
            }

            if self.ball_position.y > Self::AREA.y_max {
                self.ball_position.y = Self::AREA.y_max;
                self.ball_velocity.y = 0.0;
            }

            if self.ball_position.y < Self::AREA.y_min {
                self.ball_position.y = Self::AREA.y_min;
                self.ball_velocity.y = 0.0;
            }
        }
    }
}

impl<D> Draw<D> for AccelerationMode<'_>
where
    D: DrawTarget,
{
    fn draw_with_style(
        &self,
        style: &crate::app::AppStyle<<D as embedded_graphics::prelude::DrawTarget>::Color>,
        draw_area: embedded_graphics::primitives::Rectangle,
        target: &mut D,
    ) -> Result<(), <D as embedded_graphics::prelude::DrawTarget>::Error> {
        if self.acceleration.is_none() {
            return Ok(());
        }

        let smallest_dimension = u32::min(draw_area.size.width, draw_area.size.height) as f32 * 0.9;

        let acc_2d = match self.acceleration {
            Some(acc) => F32x2 {
                x: -acc.y,
                y: -acc.x,
            },
            None => F32x2 { x: 0.0, y: 0.0 },
        };

        let vector_style = PrimitiveStyleBuilder::new()
            .stroke_width(1)
            .stroke_color(style.default_color)
            .stroke_alignment(StrokeAlignment::Center)
            .build();

        let vector = Vector::new(Point::zero(), Point::new(acc_2d.x as i32, acc_2d.y as i32))
            .into_styled(vector_style);

        // Frame with ball
        let frame_height = draw_area.size.height as f32 * 0.9;

        let scale = (frame_height - Self::BALL_DIAMETER as f32) / Self::AREA.height();
        let scaled_position = self.ball_position.mul(scale);
        let offset = Point::new(scaled_position.x as i32, scaled_position.y as i32);

        let frame_style = PrimitiveStyleBuilder::new()
            .stroke_color(style.default_color)
            .stroke_width(1)
            .stroke_alignment(StrokeAlignment::Center)
            .build();

        let frame = Rectangle::new(
            Point::zero(),
            Size::new(frame_height as u32, frame_height as u32),
        )
        .into_styled(frame_style);

        let ball_style = PrimitiveStyleBuilder::new()
            .fill_color(style.default_color)
            .build();

        let ball = Circle::new(Point::zero(), Self::BALL_DIAMETER).into_styled(ball_style);

        let ball_in_frame = CenteredWithOffset::new(frame, ball, offset).arrange();

        let layout = LinearLayout::horizontal(Chain::new(vector).append(ball_in_frame))
            .with_spacing(FixedMargin(5))
            .with_alignment(vertical::Top)
            .arrange()
            .align_to(&draw_area, horizontal::Center, vertical::Center);

        layout.draw(target)?;

        Ok(())
    }
}

#[async_trait]
impl<D> AppMode<D> for AccelerationMode<'_>
where
    D: DrawTarget,
{
    fn title(&self) -> String {
        String::from("Acceleration")
    }
}
