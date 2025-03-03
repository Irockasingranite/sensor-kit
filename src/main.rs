#![no_std]
#![no_main]

use core::fmt::write;
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
    text::{Alignment, Text},
};
use embedded_hal_bus::{i2c as i2c_bus, util::AtomicCell};
use heapless::String;
use ssd1315::Ssd1315;
use u8g2_fonts::{fonts::u8g2_font_6x10_tf, U8g2TextStyle};
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

    let text_style = U8g2TextStyle::new(u8g2_font_6x10_tf, BinaryColor::On);

    loop {
        if let Ok(reading) = dht20.read() {
            let mut temperature_display = String::<32>::new();
            if let Ok(()) = write(
                &mut temperature_display,
                format_args!("Temperature: {:.2}°C", reading.temperature),
            ) {
                _ = Text::with_alignment(
                    &temperature_display,
                    Point::new(5, 15),
                    &text_style,
                    Alignment::Left,
                )
                .draw(&mut display);
            }

            let mut humidity_display = String::<32>::new();
            if let Ok(()) = write(
                &mut humidity_display,
                format_args!("Humidity: {:.2}%", reading.humidity),
            ) {
                _ = Text::with_alignment(
                    &humidity_display,
                    Point::new(5, 30),
                    &text_style,
                    Alignment::Left,
                )
                .draw(&mut display);
            }
        }

        display.flush_screen();

        Timer::after_secs(1).await;
    }
}
