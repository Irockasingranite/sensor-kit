/// Environment sensors.
mod environment;
/// Potentiometer.
mod potentiometer;

pub use environment::SensorKitEnvSensors;
pub use potentiometer::SensorKitPotentiometer;

use thiserror::Error;

#[derive(Debug, Error)]
/// Error type used by peripheral structs.
pub enum PeripheralError {
    #[error("Error during I2C transaction")]
    /// An I2C error.
    I2cError,
}
