//! # autocore-sitl
//!
//! AutoCore Otonom Sistem Mimarisi için Masaüstü SITL (Software-in-the-Loop)
//! Simülasyon Tezgâhı. Sanal PTY terminalleri (`/dev/pts/*`), fiziksel
//! atalet modelli Mock Aktüatör ve sanal CAN veri yolları.

pub mod mock_actuator;
pub mod pty;
pub mod sim_bridge;
pub mod vcan;

pub use mock_actuator::MockActuator;
pub use pty::VirtualPtyPair;
pub use sim_bridge::SitlSerialBridge;
pub use vcan::{VirtualCanBus, VirtualCanEndpoint};

#[cfg(test)]
mod tests {
    use super::*;
    use autocore_base::KinematicCommand;
    use autocore_hal::ActuatorPort;

    #[tokio::test]
    async fn test_sitl_workspace_integration() {
        // Mock aktüatör ile temel kinematik doğrulama
        let mut actuator = MockActuator::default();
        let cmd = KinematicCommand::new([2.0, 0.0, 0.0], [0.0; 3], 100);
        actuator.send_command(&cmd).await.unwrap();

        actuator.step(200);
        let feedback = actuator.read_feedback().await.unwrap();
        assert!(feedback.actual_linear[0] > 0.0);
    }
}
