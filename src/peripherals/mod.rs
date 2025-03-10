/// Analog Input.
mod analog_input;
/// Environment sensors.
mod environment;
/// PWM driven LED.
mod pwm_led;

pub use analog_input::{AnalogInput, ReversedAnalogInput};
pub use environment::SensorKitEnvSensors;
pub use pwm_led::PwmLed;

use thiserror::Error;

#[derive(Debug, Error)]
/// Error type used by peripheral structs.
pub enum PeripheralError {
    #[error("Error during I2C transaction")]
    /// An I2C error.
    I2cError,
    #[error("PWM error")]
    /// A PWM error.
    PwmError,
}
