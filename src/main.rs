#![no_std]
#![no_main]

extern crate alloc;

mod app;
mod mode;
mod peripherals;
mod ui;

use app::{AppMode, AppStyle};
use mode::{
    environment::EnvironmentMode,
    potentiometer::PotentiometerMode,
};
use peripherals::{SensorKitEnvSensors, SensorKitPotentiometer};
use ui::TitleFrame;

use alloc::{boxed::Box, vec, vec::Vec};
use bme280::i2c::BME280;
use core::mem::MaybeUninit;
use display_interface_i2c::I2CInterface;
use embassy_executor::Spawner;
use embassy_stm32::{
    adc::Adc,
    bind_interrupts,
    i2c::{self, I2c},
    time::Hertz,
};
use embassy_time::{Delay, Duration, Instant, Timer};
use embedded_alloc::LlffHeap as Heap;
use embedded_dht_rs::dht20::Dht20;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use embedded_hal_bus::{
    i2c::self as i2c_bus,
    util::AtomicCell,
};
use ssd1315::Ssd1315;
use u8g2_fonts::{fonts, U8g2TextStyle};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_ER => i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;
    I2C1_EV => i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>;
});

#[global_allocator]
static HEAP: Heap = Heap::empty();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    const HEAP_SIZE: usize = 4096;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    #[allow(static_mut_refs)]
    unsafe {
        HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE)
    }

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

    let dht20 = Dht20::new(i2c_bus::AtomicDevice::new(&i2c1), Delay);
    let mut bmp280 = BME280::new_secondary(i2c_bus::AtomicDevice::new(&i2c1));
    let display_interface = I2CInterface::new(i2c_bus::AtomicDevice::new(&i2c1), 0x3c, 0b01000000);
    let mut display = Ssd1315::new(display_interface);

    bmp280
        .init(&mut Delay)
        .expect("Failed to initialize BMP280)");

    display.init_screen();
    display.flush_screen();

    let sensors = SensorKitEnvSensors::new(dht20, bmp280);

    let text_style = U8g2TextStyle::new(fonts::u8g2_font_mercutio_basic_nbp_tf, BinaryColor::On);
    let title_style = U8g2TextStyle::new(fonts::u8g2_font_mercutio_sc_nbp_tf, BinaryColor::On);
    let app_style = AppStyle::new(title_style, text_style, BinaryColor::On);

    let adc = Adc::new(p.ADC1);
    let adc_channel = p.PA3;
    let potentiometer = SensorKitPotentiometer::new(adc, adc_channel);

    let environment_mode = EnvironmentMode::new(sensors);
    let potentiometer_mode = PotentiometerMode::new(potentiometer);

    let mut modes: Vec<Box<dyn AppMode<_>>> =
        vec![Box::new(environment_mode), Box::new(potentiometer_mode)];

    loop {
        for mode in modes.iter_mut() {
            let mode_start = Instant::now();
            loop {
                if Instant::now() - mode_start > Duration::from_secs(5) {
                    break;
                }

                mode.update();
                let title = mode.title();

                let frame = TitleFrame::new(
                    &title,
                    app_style.title_style.clone(),
                    BinaryColor::On,
                    display.bounding_box(),
                );

                let inner_area = frame.inner_area();

                _ = frame.draw(&mut display);
                _ = mode.draw_with_style(&app_style, inner_area, &mut display);

                display.flush_screen();

                Timer::after_millis(100).await;
            }
        }
    }
}
