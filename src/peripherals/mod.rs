mod environment;
mod potentiometer;

pub use environment::SensorKitEnvSensors;
pub use potentiometer::SensorKitPotentiometer;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PeripheralError {
    #[error("Error during I2C transaction")]
    I2cError,
}
