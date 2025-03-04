#![no_std]
#![no_main]

use core::fmt::Write;
use display_interface_i2c::I2CInterface;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    i2c::{self, I2c},
    peripherals,
    time::Hertz,
};
use embassy_time::{Delay, Timer};
use embedded_dht_rs::dht20::Dht20;
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use embedded_hal_bus::{i2c as i2c_bus, util::AtomicCell};
use embedded_layout::layout::linear::{spacing::DistributeFill, FixedMargin, LinearLayout};
use embedded_layout::prelude::*;
use heapless::String;
use ssd1315::Ssd1315;
use u8g2_fonts::U8g2TextStyle;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_ER => i2c::ErrorInterruptHandler<peripherals::I2C1>;
    I2C1_EV => i2c::EventInterruptHandler<peripherals::I2C1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let i2c1 = I2c::new(
        p.I2C1,
        p.PB8,
        p.PB9,
        Irqs,
        p.DMA1_CH1,
        p.DMA1_CH0,
        Hertz::khz(400),
        Default::default(),
    );

    let i2c1 = AtomicCell::new(i2c1);

    let mut dht20 = Dht20::new(i2c_bus::AtomicDevice::new(&i2c1), Delay);
    let display_interface = I2CInterface::new(i2c_bus::AtomicDevice::new(&i2c1), 0x3c, 0b01000000);
    let mut display = Ssd1315::new(display_interface);

    display.init_screen();
    display.flush_screen();

    loop {
        if let Ok(reading) = dht20.read() {
            let bounds = display.bounding_box();
            let frame =
                draw_frame_with_title("Temp. & Hum.", &mut display, bounds, BinaryColor::On)
                    .unwrap();

            draw_temp_hum(
                reading.temperature,
                reading.humidity,
                &mut display,
                frame,
                BinaryColor::On,
            )
            .unwrap();
        }

        display.flush_screen();

        Timer::after_secs(1).await;
    }
}

fn draw_frame_with_title<D, C, E>(
    title: &str,
    target: &mut D,
    bounds: Rectangle,
    color: C,
) -> Result<Rectangle, E>
where
    D: DrawTarget<Color = C, Error = E>,
    C: PixelColor,
{
    let font = u8g2_fonts::fonts::u8g2_font_6x10_tf;
    let text_style = U8g2TextStyle::new(font, color);
    let frame_style = PrimitiveStyleBuilder::new()
        .stroke_color(color)
        .stroke_width(1)
        .build();

    let title = Text::new(title, Point::zero(), text_style).align_to(
        &bounds,
        horizontal::Center,
        vertical::Top,
    );

    let title_height = title.bounding_box().size.height;
    let remaining_size = bounds.size - Size::new(0, title_height);

    let remaining_area = Rectangle::new(Point::zero(), remaining_size).align_to(
        &bounds,
        horizontal::NoAlignment,
        vertical::Bottom,
    );
    let frame = remaining_area.offset(-1);

    title.draw(target)?;
    frame.into_styled(frame_style).draw(target)?;

    Ok(frame.offset(-3))
}

fn draw_temp_hum<D, C, E>(
    temperature_c: f32,
    humidity_pct: f32,
    target: &mut D,
    bounds: Rectangle,
    color: C,
) -> Result<(), E>
where
    D: DrawTarget<Color = C, Error = E>,
    C: PixelColor,
{
    let font = u8g2_fonts::fonts::u8g2_font_6x13_mf;
    let style = U8g2TextStyle::new(font, color);

    let mut temp_str = String::<8>::new();
    _ = write!(&mut temp_str, "{:.2}°C", temperature_c);

    let mut hum_str = String::<8>::new();
    _ = write!(&mut hum_str, "{:.2} %", humidity_pct);

    let temp_label = Text::new("Temperature:", Point::zero(), &style);
    let temp_value = Text::new(&temp_str, Point::zero(), &style);
    let hum_label = Text::new("Humidity:", Point::zero(), &style);
    let hum_value = Text::new(&hum_str, Point::zero(), &style).align_to(
        &bounds,
        horizontal::NoAlignment,
        vertical::Top,
    );

    let temp_line = LinearLayout::horizontal(Chain::new(temp_label).append(temp_value))
        .with_spacing(DistributeFill(bounds.size.width))
        .arrange();

    let hum_line = LinearLayout::horizontal(Chain::new(hum_label).append(hum_value))
        .with_spacing(DistributeFill(bounds.size.width))
        .arrange();

    let all_lines = LinearLayout::vertical(Chain::new(temp_line).append(hum_line))
        .with_spacing(FixedMargin(5))
        .arrange()
        .align_to(&bounds, horizontal::Left, vertical::Center);

    all_lines.draw(target)?;

    Ok(())
}
