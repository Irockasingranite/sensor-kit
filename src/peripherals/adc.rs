use super::PeripheralError;

use alloc::boxed::Box;
use alloc::sync::Arc;
use async_trait::async_trait;
use embassy_sync::blocking_mutex::raw::RawMutex;
use embassy_sync::mutex::Mutex;

#[async_trait]
/// An analog input.
pub trait AnalogInput: Send {
    /// Raw input value.
    async fn input_raw(&mut self) -> Result<u16, PeripheralError>;

    /// Maximum value this input can provide.
    async fn max_value(&self) -> Result<u16, PeripheralError>;

    /// Relative input value as compared to its maximum value.
    async fn input_pct(&mut self) -> Result<f32, PeripheralError> {
        let raw = self.input_raw().await?;
        let max = self.max_value().await?;
        let pct = (raw as f32 / max as f32) * 100.0;
        Ok(pct)
    }
}

#[async_trait]
impl<T, M> AnalogInput for Arc<Mutex<M, T>>
where
    T: AnalogInput + Send,
    M: RawMutex + Send + Sync,
{
    async fn input_raw(&mut self) -> Result<u16, PeripheralError> {
        let mut lock = self.lock().await;
        lock.input_raw().await
    }

    async fn max_value(&self) -> Result<u16, PeripheralError> {
        let lock = self.lock().await;
        lock.max_value().await
    }

    async fn input_pct(&mut self) -> Result<f32, PeripheralError> {
        let mut lock = self.lock().await;
        lock.input_pct().await
    }
}

/// An analog input whose relative value is reversed (i.e. the maximum value registers as 0%, and 0
/// registers as 100%).
pub struct ReversedAnalogInput<T> {
    input: T,
}

impl<T> ReversedAnalogInput<T> {
    /// Create a new reversed input based on an existing input.
    pub fn new(input: T) -> Self {
        Self { input }
    }
}

#[async_trait]
impl<T> AnalogInput for ReversedAnalogInput<T>
where
    T: AnalogInput + Sync,
{
    async fn input_raw(&mut self) -> Result<u16, PeripheralError> {
        self.input.input_raw().await
    }

    async fn max_value(&self) -> Result<u16, PeripheralError> {
        self.input.max_value().await
    }

    async fn input_pct(&mut self) -> Result<f32, PeripheralError> {
        let pct = self.input.input_pct().await?;
        Ok(100.0 - pct)
    }
}
