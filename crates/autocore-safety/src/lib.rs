//! # autocore-safety
//!
//! SANCAK Otonom Sistem Mimarisi için Emniyet Sentineli, 7-Durumlu FSM,
//! 500ms Watchdog, Dinamik Geofence Clamping, Slew-Rate Limiter ve
//! Hayalet Hareket Koruması (Phantom Movement Fix).

pub mod clamping;
pub mod error;
pub mod flush;
pub mod fsm;
pub mod slew_rate;
pub mod watchdog;

pub use clamping::ClampingEngine;
pub use error::SafetyError;
pub use flush::LockFreeFlushQueue;
pub use fsm::CentralSafetyFsm;
pub use slew_rate::SlewRateLimiter;
pub use watchdog::SentinelWatchdog;

#[cfg(test)]
mod tests {
    use super::*;
    use autocore_base::{KinematicCommand, SafetyMode};

    #[test]
    fn test_safety_core_integration() {
        let fsm = CentralSafetyFsm::new(SafetyMode::Booting);
        let watchdog = SentinelWatchdog::new(500_000);
        let clamping = ClampingEngine::default();
        let mut slew_rate = SlewRateLimiter::default();
        let queue = LockFreeFlushQueue::<KinematicCommand>::new(10);

        // 1. FSM Booting -> Disarmed -> Armed -> Mission
        fsm.transition(SafetyMode::Disarmed).unwrap();
        fsm.transition(SafetyMode::Armed).unwrap();
        fsm.transition(SafetyMode::Mission).unwrap();
        assert_eq!(fsm.current_mode(), SafetyMode::Mission);

        // 2. Watchdog feed
        watchdog.feed(1_000_000);
        assert!(watchdog.check_timeout(1_200_000, &fsm).is_ok());

        // 3. Komut üretimi ve sınırlaması
        let mut raw_cmd = KinematicCommand::new([25.0, 0.0, 0.0], [0.0; 3], 1_200_000);
        clamping.clamp_command(&mut raw_cmd);
        assert_eq!(raw_cmd.linear[0], 15.0); // max 15 m/s

        let smoothed_cmd = slew_rate.filter(raw_cmd, 1_200_000);
        queue.push(smoothed_cmd);
        assert_eq!(queue.len(), 1);

        // 4. Watchdog aşımı simülasyonu (600ms gecikme)
        let res = watchdog.check_timeout(1_800_001, &fsm);
        assert!(res.is_err());
        assert_eq!(fsm.current_mode(), SafetyMode::SafeLock);

        // 5. Kuyruk temizliği (Hayalet Hareket Koruması)
        queue.flush_all();
        assert_eq!(queue.len(), 0);
    }
}
