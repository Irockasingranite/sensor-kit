/// Acceleration mode
pub mod acceleration;
/// Buzzer mode.
pub mod buzzer;
/// Environment mode.
pub mod environment;
/// LED mode.
pub mod led;
/// Light sensor mode.
pub mod light;
/// Potentiometer mode.
pub mod potentiometer;
/// Sound mode.
pub mod sound;

pub use environment::EnvironmentMode;
pub use led::LedMode;
pub use light::LightSensorMode;
pub use potentiometer::PotentiometerMode;
pub use sound::SoundMode;
pub use acceleration::AccelerationMode;

pub mod inputs {
    use crate::peripherals::PeripheralError;
    use alloc::boxed::Box;
    use alloc::sync::Arc;
    use async_trait::async_trait;
    use embassy_sync::{blocking_mutex::raw::RawMutex, mutex::Mutex};

    #[async_trait]
    pub trait PctInput: Send {
        async fn input_pct(&mut self) -> Result<f32, PeripheralError>;
    }

    #[async_trait]
    impl<T, M> PctInput for Arc<Mutex<M, T>>
    where
        T: PctInput + Send,
        M: RawMutex + Send + Sync,
    {
        async fn input_pct(&mut self) -> Result<f32, PeripheralError> {
            let mut input = self.lock().await;
            input.input_pct().await
        }
    }
}
