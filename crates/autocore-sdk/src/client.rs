//! autocore-sdk :: İstemci Kiti ve Görev Yöneticisi (AutocoreClient)
//!
//! Takımların otonom algoritmalarını (Python, C++ veya Rust) Autocore çekirdeğine
//! ve SITL simülasyonuna bağlayan yüksek seviyeli asenkron arayüz.

use crate::blackbox::BlackboxRecorder;
use autocore_base::{ActuatorFeedback, HalError, KinematicCommand, SafetyMode, StateVector6DoF};
use autocore_hal::serial::AsyncStreamActuator;
use autocore_hal::traits::ActuatorPort;
use autocore_sitl::sim_bridge::SitlSerialBridge;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tracing::info;

pub struct AutocoreClient {
    port: Box<dyn ActuatorPort>,
    current_mode: Arc<AtomicU8>,
    blackbox: Option<BlackboxRecorder>,
    sitl_bridge: Option<SitlSerialBridge>,
}

impl AutocoreClient {
    /// Herhangi bir `ActuatorPort` üzerinden istemci başlatır
    pub fn new(port: Box<dyn ActuatorPort>) -> Self {
        Self {
            port,
            current_mode: Arc::new(AtomicU8::new(SafetyMode::Booting as u8)),
            blackbox: None,
            sitl_bridge: None,
        }
    }

    /// Tamamen yerel, donanımsız masaüstü SITL simülasyon ortamına bağlanır
    pub async fn connect_sitl() -> Result<Self, HalError> {
        info!("SITL Köprüsü başlatılıyor...");
        let bridge = SitlSerialBridge::start()?;
        let slave_path = bridge.serial_port_path();
        info!("Sanal PTY bağlandı: {:?}", slave_path);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(slave_path)
            .await
            .map_err(|e| HalError::SerialError(e.to_string()))?;

        let actuator = AsyncStreamActuator::new(file);

        Ok(Self {
            port: Box::new(actuator),
            current_mode: Arc::new(AtomicU8::new(SafetyMode::Disarmed as u8)),
            blackbox: None,
            sitl_bridge: Some(bridge),
        })
    }

    /// SITL köprü referansı (aktifse)
    pub fn sitl_bridge(&self) -> Option<&SitlSerialBridge> {
        self.sitl_bridge.as_ref()
    }

    /// Siyah kutu kaydedicisini etkinleştirir
    pub fn enable_blackbox(&mut self, capacity: usize) {
        self.blackbox = Some(BlackboxRecorder::new(capacity));
    }

    /// Siyah kutu referansı
    pub fn blackbox(&self) -> Option<&BlackboxRecorder> {
        self.blackbox.as_ref()
    }

    /// Siyah kutu mut referansı
    pub fn blackbox_mut(&mut self) -> Option<&mut BlackboxRecorder> {
        self.blackbox.as_mut()
    }

    /// Mevcut emniyet modunu döndürür
    pub fn safety_mode(&self) -> SafetyMode {
        SafetyMode::from_u8(self.current_mode.load(Ordering::SeqCst))
            .unwrap_or(SafetyMode::SafeLock)
    }

    /// Emniyet modunu değiştirir (Disarmed -> Armed / Mission)
    pub fn set_safety_mode(&mut self, mode: SafetyMode) {
        self.current_mode.store(mode as u8, Ordering::SeqCst);
    }

    /// Kinematik hız komutu gönderir (Vx, Vy, Vz, RollRate, PitchRate, YawRate)
    pub async fn send_cmd_vel(
        &mut self,
        linear: [f32; 3],
        angular: [f32; 3],
        timestamp_us: u64,
    ) -> Result<(), HalError> {
        let cmd = KinematicCommand::new(linear, angular, timestamp_us);
        self.send_command(&cmd).await
    }

    /// Doğrudan `KinematicCommand` nesnesi gönderir
    pub async fn send_command(&mut self, cmd: &KinematicCommand) -> Result<(), HalError> {
        self.port.send_command(cmd).await?;

        // Geri beslemeyi oku ve varsa siyah kutuya kaydet
        if let Ok(fb) = self.port.read_feedback().await {
            if let Some(ref mut bbox) = self.blackbox {
                let state = StateVector6DoF {
                    velocity: [cmd.linear[0], cmd.linear[1], cmd.linear[2]],
                    angular_rates: [cmd.angular[0], cmd.angular[1], cmd.angular[2]],
                    timestamp_us: cmd.timestamp_us,
                    ..Default::default()
                };
                bbox.record(cmd.timestamp_us, state, *cmd, fb);
            }
        }
        Ok(())
    }

    /// Aktüatörlerden son telemetri geri beslemesini çeker
    pub async fn read_feedback(&mut self) -> Result<ActuatorFeedback, HalError> {
        self.port.read_feedback().await
    }

    /// Acil Durdurma (E-Stop) emri gönderir
    pub async fn emergency_stop(&mut self) -> Result<(), HalError> {
        self.current_mode.store(SafetyMode::SafeLock as u8, Ordering::SeqCst);
        self.port.emergency_stop().await
    }

    /// Bağlantının açık olup olmadığını kontrol eder
    pub fn is_connected(&self) -> bool {
        self.port.is_connected()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_sitl_lifecycle() {
        let mut client = AutocoreClient::connect_sitl().await.unwrap();
        assert!(client.is_connected());
        assert_eq!(client.safety_mode(), SafetyMode::Disarmed);

        // Blackbox aç
        client.enable_blackbox(10);

        // Arm et ve görev moduna al
        client.set_safety_mode(SafetyMode::Mission);
        assert_eq!(client.safety_mode(), SafetyMode::Mission);

        // Komut gönder
        for i in 1..=3 {
            client
                .send_cmd_vel([2.0, 0.0, 0.0], [0.0, 0.0, 0.1 * i as f32], i * 20000)
                .await
                .unwrap();
        }

        // Geri besleme oku
        let fb = client.read_feedback().await.unwrap();
        assert_eq!(fb.status_flags, 0);

        // Blackbox kaydını doğrula
        let bbox = client.blackbox().unwrap();
        let frames = bbox.ordered_frames();
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].command.linear[0], 2.0);

        // Acil durdurma tetikle
        client.emergency_stop().await.unwrap();
        assert_eq!(client.safety_mode(), SafetyMode::SafeLock);
    }
}
