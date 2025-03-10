use alloc::boxed::Box;
use alloc::string::String;
use async_trait::async_trait;
use core::fmt::Write;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::Text;
use embedded_layout::layout::linear::spacing::DistributeFill;
use embedded_layout::layout::linear::{FixedMargin, LinearLayout};
use embedded_layout::prelude::*;

use crate::app::{AppMode, AppStyle};
use crate::app::{Draw, Update};
use crate::peripherals::PeripheralError;

/// Struct defining the 'Environment' mode. Samples data from a set of sensors and displays it in a
/// table.
pub struct EnvironmentMode<'a> {
    /// Sensors used as data source.
    sensors: Box<dyn EnvironmentSensors + 'a>,
    /// Temperature in °C.
    temperature_c: Option<f32>,
    /// Humidity in %.
    humidity_pct: Option<f32>,
    /// Pressure in kPa.
    pressure_kpa: Option<f32>,
}

impl<'a> EnvironmentMode<'a> {
    pub fn new(sensors: impl EnvironmentSensors + 'a) -> Self {
        let sensors = Box::new(sensors);

        Self {
            sensors,
            temperature_c: None,
            humidity_pct: None,
            pressure_kpa: None,
        }
    }
}

#[async_trait]
impl Update for EnvironmentMode<'_> {
    async fn update(&mut self) {
        self.temperature_c = self.sensors.get_temperature().ok();
        self.humidity_pct = self.sensors.get_humidity().ok();
        self.pressure_kpa = self.sensors.get_pressure().ok();
    }
}

impl<D> Draw<D> for EnvironmentMode<'_>
where
    D: DrawTarget,
{
    fn draw_with_style(
        &self,
        style: &AppStyle<D::Color>,
        draw_area: Rectangle,
        target: &mut D,
    ) -> Result<(), D::Error> {
        let mut temp_str = String::new();
        if let Some(temperature) = self.temperature_c {
            _ = write!(&mut temp_str, "{:.2}°C", temperature);
        } else {
            _ = write!(&mut temp_str, "???");
        }

        let mut hum_str = String::new();
        if let Some(humidity) = self.humidity_pct {
            _ = write!(&mut hum_str, "{:.2}%", humidity);
        } else {
            _ = write!(&mut hum_str, "???");
        }

        let mut pres_str = String::new();
        if let Some(pressure) = self.pressure_kpa {
            _ = write!(&mut pres_str, "{:.1}kPa", pressure);
        } else {
            _ = write!(&mut pres_str, "???");
        }

        let temp_label = Text::new("Temp.:", Point::zero(), style.text_style.clone());
        let temp_value = Text::new(&temp_str, Point::zero(), style.text_style.clone());
        let hum_label = Text::new("Humidity:", Point::zero(), style.text_style.clone());
        let hum_value = Text::new(&hum_str, Point::zero(), style.text_style.clone());
        let pres_label = Text::new("Pressure:", Point::zero(), style.text_style.clone());
        let pres_value = Text::new(&pres_str, Point::zero(), style.text_style.clone());

        let temp_line = LinearLayout::horizontal(Chain::new(temp_label).append(temp_value))
            .with_spacing(DistributeFill(draw_area.size.width))
            .arrange();

        let hum_line = LinearLayout::horizontal(Chain::new(hum_label).append(hum_value))
            .with_spacing(DistributeFill(draw_area.size.width))
            .arrange();

        let pres_line = LinearLayout::horizontal(Chain::new(pres_label).append(pres_value))
            .with_spacing(DistributeFill(draw_area.size.width))
            .arrange();

        let all_lines =
            LinearLayout::vertical(Chain::new(temp_line).append(hum_line).append(pres_line))
                .with_spacing(FixedMargin(2))
                .arrange()
                .align_to(&draw_area, horizontal::Center, vertical::Center);

        all_lines.draw(target)
    }
}

#[async_trait]
impl<D> AppMode<D> for EnvironmentMode<'_>
where
    D: DrawTarget,
{
    fn title(&self) -> String {
        String::from("Environment")
    }
}

/// Defines interface for environment sensors that can be used by the [`EnvironmentMode`].
pub trait EnvironmentSensors: Send {
    /// Return the current temperature in °C.
    fn get_temperature(&mut self) -> Result<f32, PeripheralError>;
    /// Return the current humidity in %.
    fn get_humidity(&mut self) -> Result<f32, PeripheralError>;
    /// Return the current air pressure in kPa.
    fn get_pressure(&mut self) -> Result<f32, PeripheralError>;
}
