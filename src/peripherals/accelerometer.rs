use accelerometer::{vector::F32x3, Accelerometer};
use alloc::boxed::Box;
use async_trait::async_trait;
use core::fmt::Debug;
use embedded_hal::i2c::I2c;
use lis3dh::{Lis3dh, Lis3dhI2C};

use super::PeripheralError;
use crate::mode::acceleration::AccelerationInput;

#[async_trait]
impl<I2C, E> AccelerationInput for Lis3dh<Lis3dhI2C<I2C>>
where
    I2C: I2c<Error = E> + Send,
    E: Send + Debug,
{
    async fn accel_norm(&mut self) -> Result<F32x3, PeripheralError> {
        Accelerometer::accel_norm(self).map_err(|_| PeripheralError::I2c)
    }
}
