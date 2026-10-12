//! autocore-hal :: Donanım Soyutlama Trait'leri (HAL Core)

use async_trait::async_trait;
use autocore_base::{ActuatorFeedback, HalError, KinematicCommand};

/// Donanım Bağımsız Aktüatör ve Motor Sürücü Arayüzü
#[async_trait]
pub trait ActuatorPort: Send + Sync {
    /// Kinematik hız ve yönelim komutu gönderir
    async fn send_command(&mut self, cmd: &KinematicCommand) -> Result<(), HalError>;

    /// Aktüatörden telemetri ve geri besleme okur
    async fn read_feedback(&mut self) -> Result<ActuatorFeedback, HalError>;

    /// Donanımı anında acil durdurma (E-Stop / fren) moduna alır
    async fn emergency_stop(&mut self) -> Result<(), HalError>;

    /// Donanım hattının bağlı ve canlı olup olmadığını bildirir
    fn is_connected(&self) -> bool;
}
