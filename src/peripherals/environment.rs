use crate::mode::environment::EnvironmentSensors;
use bme280::i2c::BME280;
use embassy_time::Delay;
use embedded_dht_rs::dht20::Dht20;

use super::PeripheralError;

pub struct SensorKitEnvSensors<I, D>
where
    I: embedded_hal::i2c::I2c,
    D: embedded_hal::delay::DelayNs,
{
    dht20: Dht20<I, D>,
    bmp280: BME280<I>,
}

impl<I, D> SensorKitEnvSensors<I, D>
where
    I: embedded_hal::i2c::I2c,
    D: embedded_hal::delay::DelayNs,
{
    pub fn new(dht20: Dht20<I, D>, bmp280: BME280<I>) -> Self {
        Self { dht20, bmp280 }
    }
}

impl<I, D> EnvironmentSensors for SensorKitEnvSensors<I, D>
where
    I: embedded_hal::i2c::I2c,
    D: embedded_hal::delay::DelayNs,
{
    fn get_temperature(&mut self) -> Result<f32, PeripheralError> {
        self.dht20
            .read()
            .map(|r| r.temperature)
            .map_err(|_| PeripheralError::I2cError)
    }

    fn get_humidity(&mut self) -> Result<f32, PeripheralError> {
        self.dht20
            .read()
            .map(|r| r.humidity)
            .map_err(|_| PeripheralError::I2cError)
    }

    fn get_pressure(&mut self) -> Result<f32, PeripheralError> {
        self.bmp280
            .measure(&mut Delay)
            .map(|r| r.pressure / 1000.0)
            .map_err(|_| PeripheralError::I2cError)
    }
}
