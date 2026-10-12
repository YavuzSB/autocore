//! # autocore-base
//!
//! SANCAK Otonom Sistem Mimarisi için temel C-ABI uyumlu veri modelleri,
//! zaman damgaları, ortak hata tipleri ve sıfır-kopyalama yapıları.

pub mod abi;
pub mod error;
pub mod time;
pub mod types;

pub use error::{AutocoreError, HalError};
pub use time::now_us;
pub use types::{ActuatorFeedback, KinematicCommand, SafetyMode, StateVector6DoF};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_roundtrip() {
        let cmd = KinematicCommand::new([1.5, -0.5, 0.0], [0.0, 0.0, 0.78], 123456789);
        let bytes = cmd.as_bytes();
        assert_eq!(bytes.len(), 32);

        let parsed = KinematicCommand::try_from_bytes(bytes).expect("Pod cast failed");
        assert_eq!(parsed, &cmd);
    }

    #[test]
    fn test_safety_mode_enum() {
        assert_eq!(SafetyMode::from_u8(0), Some(SafetyMode::Booting));
        assert_eq!(SafetyMode::from_u8(5), Some(SafetyMode::SafeLock));
        assert_eq!(SafetyMode::from_u8(99), None);
        assert_eq!(SafetyMode::SafeLock.as_str(), "SAFELOCK");
    }
}
