#![no_std]
#![no_main]

extern crate alloc;

mod app;
mod mode;
mod peripherals;
mod ui;

use app::{AppMode, AppStyle};
use mode::potentiometer::PotentiometerMode;
use mode::{environment::EnvironmentMode, light::LightSensorMode};
use peripherals::{SensorKitEnvSensors, SensorKitLightSensor, SensorKitPotentiometer};
use ui::TitleFrame;

use alloc::vec;
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use bme280::i2c::BME280;
use core::mem::MaybeUninit;
use display_interface_i2c::I2CInterface;
use embassy_executor::{task, Spawner};
use embassy_stm32::{
    adc::Adc,
    bind_interrupts,
    exti::{self, ExtiInput},
    gpio,
    i2c::{self, I2c},
    time::Hertz,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex, signal::Signal};
use embassy_time::{Delay, Timer};
use embedded_alloc::LlffHeap as Heap;
use embedded_dht_rs::dht20::Dht20;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use embedded_hal_bus::{i2c as i2c_bus, util::AtomicCell};
use ssd1315::Ssd1315;
use u8g2_fonts::{fonts, U8g2TextStyle};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C1_ER => i2c::ErrorInterruptHandler<embassy_stm32::peripherals::I2C1>;
    I2C1_EV => i2c::EventInterruptHandler<embassy_stm32::peripherals::I2C1>;
});

#[global_allocator]
static HEAP: Heap = Heap::empty();

static BUTTON_SIGNAL: Signal<CriticalSectionRawMutex, bool> = Signal::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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

    // Set up peripherals

    // Button used for mode switching
    let button = exti::ExtiInput::new(p.PF14, p.EXTI14, gpio::Pull::Down);

    // I2C bus
    let i2c1 = AtomicCell::new(i2c1);

    // Environment sensors
    let dht20 = Dht20::new(i2c_bus::AtomicDevice::new(&i2c1), Delay);
    let mut bmp280 = BME280::new_secondary(i2c_bus::AtomicDevice::new(&i2c1));
    bmp280
        .init(&mut Delay)
        .expect("Failed to initialize BMP280)");

    // Potentiometer ADC
    let adc = Adc::new(p.ADC1);
    let adc = Arc::new(Mutex::<CriticalSectionRawMutex, _>::new(adc));

    let potentiometer_adc_channel = p.PA3;
    let light_sensor_adc_channel = p.PC1;

    // Display
    let display_interface = I2CInterface::new(i2c_bus::AtomicDevice::new(&i2c1), 0x3c, 0b01000000);
    let mut display = Ssd1315::new(display_interface);
    display.init_screen();
    display.flush_screen();

    // Define App style
    let text_style = U8g2TextStyle::new(fonts::u8g2_font_mercutio_basic_nbp_tf, BinaryColor::On);
    let title_style = U8g2TextStyle::new(fonts::u8g2_font_mercutio_sc_nbp_tf, BinaryColor::On);
    let app_style = AppStyle::new(title_style, text_style, BinaryColor::On);

    // Set up modes

    // Environment mode
    let sensors = SensorKitEnvSensors::new(dht20, bmp280);
    let environment_mode = EnvironmentMode::new(sensors);

    // Potentiometer mode
    let potentiometer = SensorKitPotentiometer::new(adc.clone(), potentiometer_adc_channel);
    let potentiometer_mode = PotentiometerMode::new(potentiometer);

    // Light sensor mode.
    let light_sensor = SensorKitLightSensor::new(adc.clone(), light_sensor_adc_channel);
    let light_mode = LightSensorMode::new(light_sensor);

    let mut modes: Vec<Box<dyn AppMode<_>>> = vec![
        Box::new(light_mode),
        Box::new(environment_mode),
        Box::new(potentiometer_mode),
    ];

    // Spawn ancillary tasks
    _ = spawner.spawn(button_handler(button, &BUTTON_SIGNAL));

    // Loop over all modes (.cycle() requires Clone, which dyn AppMode isn't)
    loop {
        for mode in modes.iter_mut() {
            let title = mode.title();

            // Continuously run the active mode
            loop {
                // If the button handler signals us, switch to the next mode
                if let Some(true) = &BUTTON_SIGNAL.try_take() {
                    break;
                }

                // Update state
                mode.update().await;

                // Set up frame with mode title
                let frame = TitleFrame::new(
                    &title,
                    app_style.title_style.clone(),
                    BinaryColor::On,
                    display.bounding_box(),
                );

                let inner_area = frame.inner_area();

                // Draw both frame and content
                _ = frame.draw(&mut display);
                _ = mode.draw_with_style(&app_style, inner_area, &mut display);
                display.flush_screen();

                Timer::after_millis(100).await;
            }
        }
    }
}

#[task]
/// Task that signals button presses.
async fn button_handler(
    mut button: ExtiInput<'static>,
    signal: &'static Signal<CriticalSectionRawMutex, bool>,
) {
    loop {
        button.wait_for_high().await;
        signal.signal(true);
        Timer::after_millis(200).await; // Interval before a new event is signaled
    }
}
