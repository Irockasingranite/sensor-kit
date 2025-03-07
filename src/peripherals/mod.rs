/// Analog Input.
mod analog_input;
/// Environment sensors.
mod environment;

pub use analog_input::AnalogInput;
pub use environment::SensorKitEnvSensors;

use thiserror::Error;

#[derive(Debug, Error)]
/// Error type used by peripheral structs.
pub enum PeripheralError {
    #[error("Error during I2C transaction")]
    /// An I2C error.
    I2cError,
}
