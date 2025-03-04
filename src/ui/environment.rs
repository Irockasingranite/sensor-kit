use core::fmt::Write;
use embedded_graphics::{
    prelude::*,
    primitives::Rectangle,
    text::{renderer::TextRenderer, Text},
};
use embedded_layout::{
    layout::linear::{spacing::DistributeFill, FixedMargin, LinearLayout},
    prelude::*,
};
use heapless::String;

pub struct TemperatureHumidityDisplay<S> {
    temperature_c: f32,
    humidity_pct: f32,
    text_style: S,
    area: Rectangle,
}

impl<S> TemperatureHumidityDisplay<S> {
    pub fn new(temperature_c: f32, humidity_pct: f32, text_style: S, area: Rectangle) -> Self {
        Self {
            temperature_c,
            humidity_pct,
            text_style,
            area,
        }
    }
}

impl<S> View for TemperatureHumidityDisplay<S> {
    fn translate_impl(&mut self, by: Point) {
        self.area.translate_impl(by);
    }

    fn bounds(&self) -> Rectangle {
        self.area
    }
}

impl<S, C> Drawable for TemperatureHumidityDisplay<S>
where
    C: PixelColor,
    S: TextRenderer<Color = C> + Clone,
{
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut temp_str = String::<8>::new();
        _ = write!(&mut temp_str, "{:.2}°C", self.temperature_c);

        let mut hum_str = String::<8>::new();
        _ = write!(&mut hum_str, "{:.2}%", self.humidity_pct);

        let mut pres_str = String::<8>::new();
        _ = write!(&mut pres_str, "??? kPa");

        let temp_label = Text::new("Temp.:", Point::zero(), self.text_style.clone());
        let temp_value = Text::new(&temp_str, Point::zero(), self.text_style.clone());
        let hum_label = Text::new("Humidity:", Point::zero(), self.text_style.clone());
        let hum_value = Text::new(&hum_str, Point::zero(), self.text_style.clone());
        let pres_label = Text::new("Pressure:", Point::zero(), self.text_style.clone());
        let pres_value = Text::new(&pres_str, Point::zero(), self.text_style.clone());

        let temp_line = LinearLayout::horizontal(Chain::new(temp_label).append(temp_value))
            .with_spacing(DistributeFill(self.area.size.width))
            .arrange();

        let hum_line = LinearLayout::horizontal(Chain::new(hum_label).append(hum_value))
            .with_spacing(DistributeFill(self.area.size.width))
            .arrange();

        let pres_line = LinearLayout::horizontal(Chain::new(pres_label).append(pres_value))
            .with_spacing(DistributeFill(self.area.size.width))
            .arrange();

        let all_lines =
            LinearLayout::vertical(Chain::new(temp_line).append(hum_line).append(pres_line))
                .with_spacing(FixedMargin(2))
                .arrange()
                .align_to(&self.area, horizontal::Center, vertical::Center);

        all_lines.draw(target)?;
        Ok(())
    }
}
