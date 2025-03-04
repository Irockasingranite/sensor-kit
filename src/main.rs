#![no_std]
#![no_main]

mod ui;

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
    geometry::AnchorPoint, pixelcolor::BinaryColor, prelude::*, primitives::Rectangle,
};
use embedded_hal_bus::{i2c as i2c_bus, util::AtomicCell};
use ssd1315::Ssd1315;
use u8g2_fonts::{fonts, U8g2TextStyle};
use ui::TitleFrame;
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
            let text_style =
                U8g2TextStyle::new(fonts::u8g2_font_mercutio_basic_nbp_tf, BinaryColor::On);
            let temp_hum_display = ui::TemperatureHumidityDisplay::new(
                reading.temperature,
                reading.humidity,
                text_style,
                Rectangle::new(Point::zero(), Size::new(100, 50)),
            );

            let title_style =
                U8g2TextStyle::new(fonts::u8g2_font_mercutio_sc_nbp_tf, BinaryColor::On);
            let framed_display = TitleFrame::new_with_area(
                temp_hum_display,
                "Environment",
                title_style,
                display.bounding_box(),
                BinaryColor::On,
            );

            framed_display.draw(&mut display).unwrap();
        }

        display.flush_screen();

        Timer::after_millis(500).await;
    }
}
