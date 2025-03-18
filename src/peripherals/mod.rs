/// Accelerometer
mod accelerometer;
/// Analog Input.
mod adc;
/// Environment sensors.
mod environment;
/// PWM.
mod pwm;

pub use adc::{AnalogInput, ReversedAnalogInput};
pub use environment::SensorKitEnvSensors;
pub use pwm::Pwm;

use thiserror::Error;

#[allow(dead_code)]
#[derive(Debug, Error)]
/// Error type used by peripheral structs.
pub enum PeripheralError {
    #[error("Error during I2C transaction")]
    /// An I2C error.
    I2c,
    #[error("Error during ADC operation")]
    /// An ADC error.
    Adc,
    #[error("Error during PWM operation")]
    /// A PWM error.
    Pwm,
}
