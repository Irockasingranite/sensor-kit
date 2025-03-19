//! # Arduino Sensor Kit Firmware
//!
//! This application demonstrates the use of a variety of peripherals found on the Arduino Sensor
//! Kit. The button connected to pin D4 cycles through a variety of modes, some of which can be
//! controlled by the potentiometer dial connected to pin A0.
//!
//! The application is structured into 3 distinct layers: The application layer describing the
//! various modes in [`mode`], the peripheral layer in [`peripherals`] defining the abstract
//! peripheral interfaces used by the application layer, and the platform layer containing platform
//! specific implementations of those interfaces. Finally, a main loop runs and switches between
//! the different modes.
//!
//! A mode is anything that implements [`AppMode`], which apart from some bookkeeping requires
//! implementing both [`app::Update`] and [`app::Draw`]. The [`app::Update`] implementation defines
//! how a mode updates its internal state (e.g. based on sensor data), and [`app::Draw`] determines
//! what should be shown on the display during this mode, depending on the mode's internal state.
//! Defining a new mode is as simple as freely defining these two operations.

#![no_std]
#![no_main]

extern crate alloc;

mod app;
mod mode;
mod peripherals;
mod platform;
mod ui;

#[cfg(feature = "nucleo-f413zh")]
use platform::nucleo_f413zh as hw_platform;

#[cfg(feature = "rp-pico")]
use platform::rp_pico as hw_platform;

use hw_platform::{platform, I2c, PinError};

use app::{AppMode, AppStyle};
use embassy_embedded_hal::shared_bus::blocking::i2c::I2cDevice;
use mode::buzzer::BuzzerMode;
use mode::{
    AccelerationMode, EnvironmentMode, LedMode, LightSensorMode, PotentiometerMode, SoundMode,
};
use peripherals::{ReversedAnalogInput, SensorKitEnvSensors};
use platform::DynSafeWait;
use ui::TitleFrame;

use alloc::vec;
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use bme280::i2c::BME280;
use core::cell::RefCell;
use core::mem::MaybeUninit;
use display_interface_i2c::I2CInterface;
use embassy_executor::{task, Spawner};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, blocking_mutex::Mutex as BlockingMutex,
    mutex::Mutex, signal::Signal,
};
use embassy_time::{Delay, Timer};
use embedded_alloc::LlffHeap as Heap;
use embedded_dht_rs::dht20::Dht20;
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use lis3dh::Lis3dh;
use ssd1315::Ssd1315;
use static_cell::StaticCell;
use u8g2_fonts::{fonts, U8g2TextStyle};
use {defmt_rtt as _, panic_probe as _};

#[global_allocator]
static HEAP: Heap = Heap::empty();

static BUTTON_SIGNAL: Signal<CriticalSectionRawMutex, bool> = Signal::new();

static I2C_BUS: StaticCell<BlockingMutex<CriticalSectionRawMutex, RefCell<I2c>>> =
    StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("Hello world");
    const HEAP_SIZE: usize = 4096;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    #[allow(static_mut_refs)]
    unsafe {
        HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE)
    }


    let platform = platform();

    let (i2c, a0, a2, a3, d4, d5, d6) = platform.split();

    let i2c = BlockingMutex::new(RefCell::new(i2c));
    let i2c = I2C_BUS.init(i2c);

    // Set up peripherals

    // Button used for mode switching
    let button = d4;

    // Accelerometer
    let lis3dh = Lis3dh::new_i2c(I2cDevice::new(i2c), lis3dh::SlaveAddr::Alternate)
        .expect("Failed to initialize LIS3DHTR");

    // Environment sensors
    let dht20 = Dht20::new(I2cDevice::new(i2c), Delay);
    let mut bmp280 = BME280::new_secondary(I2cDevice::new(i2c));
    bmp280
        .init(&mut Delay)
        .expect("Failed to initialize BMP280)");

    // Potentiometer input
    let potentiometer = ReversedAnalogInput::new(a0);
    let potentiometer: Arc<Mutex<CriticalSectionRawMutex, _>> = Arc::new(Mutex::new(potentiometer));

    // PWM
    let pwm_led = d6;
    let buzzer_pwm = d5;

    // Display
    let display_interface = I2CInterface::new(I2cDevice::new(i2c), 0x3c, 0b01000000);
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
    let potentiometer_mode = PotentiometerMode::new(potentiometer.clone());

    // Light sensor mode.
    let light_sensor = a3;
    let light_mode = LightSensorMode::new(light_sensor);

    // Sound sensor mode.
    let sound_sensor = a2;
    let sound_mode = SoundMode::new(sound_sensor);

    // LED mode
    let led_mode = LedMode::new(pwm_led, potentiometer.clone());

    // Buzzer mode
    let buzzer_mode = BuzzerMode::new(potentiometer.clone(), buzzer_pwm);

    // Acceleration mode
    let acceleration_mode = AccelerationMode::new(lis3dh);

    let mut modes: Vec<Box<dyn AppMode<_>>> = vec![
        Box::new(environment_mode),
        Box::new(potentiometer_mode),
        Box::new(acceleration_mode),
        Box::new(light_mode),
        Box::new(sound_mode),
        Box::new(led_mode),
        Box::new(buzzer_mode),
    ];

    // Spawn ancillary tasks
    spawner
        .spawn(button_handler(Box::new(button), &BUTTON_SIGNAL))
        .unwrap();

    // Loop over all modes (.cycle() requires Clone, which dyn AppMode isn't)
    loop {
        for mode in modes.iter_mut() {
            let title = mode.title();
            _ = mode.enter().await;

            // Continuously run the active mode
            loop {
                // If the button handler signals us, switch to the next mode
                if let Some(true) = &BUTTON_SIGNAL.try_take() {
                    _ = mode.exit().await;
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
    mut button: Box<dyn DynSafeWait<Error = PinError> + 'static>,
    signal: &'static Signal<CriticalSectionRawMutex, bool>,
) {
    loop {
        if button.wait_for_high().await.is_ok() {
            signal.signal(true);
        }
        Timer::after_millis(200).await; // Interval before a new event is signaled
    }
}
