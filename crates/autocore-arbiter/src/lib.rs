//! # autocore-arbiter
//!
//! 3 Seviyeli Katı Öncelik Hakemi, Güvenlik Hiyerarşisi ve Çakışma Çözücü.
//!
//! - Tier 1: Mutlak Veto & Emniyet (E-Stop, SafeLock, Watchdog)
//! - Tier 2: Reaktif Çarpışma Önleme (DWA/APF, Costmap, FlightEnvelope)
//! - Tier 3: Otonom Görev Katmanı (Kullanıcı Algoritması)

pub mod arbiter;
pub mod mock;
pub mod rate_limiter;
pub mod types;

pub use arbiter::CommandArbiter;
pub use mock::MockArbiterHarness;
pub use rate_limiter::SlewRateLimiter;
pub use types::{ArbitratedCommand, OverrideReason, PriorityTier, Tier1SafetyStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use autocore_base::{KinematicCommand, SafetyMode};

    /// Doğrulama 1: Tier 1 (E-Stop / SafeLock) anında Tier 3 kullanıcı komutunun kesilmesi
    #[test]
    fn test_tier1_veto_preemption() {
        let mut arbiter = CommandArbiter::default();

        let mission_cmd = KinematicCommand {
            linear: [3.0, 0.0, 0.0], // 3 m/s son sürat
            angular: [0.0, 0.0, 0.5],
            timestamp_us: 1000,
        };

        // 1. Normal seyir (Tier 3)
        let tier1_ok = Tier1SafetyStatus::safe_operating();
        let cmd = arbiter.arbitrate(&tier1_ok, None, Some(mission_cmd), 0.1, 1000);
        assert_eq!(cmd.active_tier, PriorityTier::Tier3Mission as u8);

        // 2. Aniden E-Stop basılıyor (Tier 1 Veto)
        let mut tier1_estop = Tier1SafetyStatus::safe_operating();
        tier1_estop.estop_active = true;

        let cmd_estop = arbiter.arbitrate(&tier1_estop, None, Some(mission_cmd), 0.1, 2000);
        assert_eq!(cmd_estop.active_tier, PriorityTier::Tier1Safety as u8);
        assert_eq!(cmd_estop.override_reason, OverrideReason::EStop as u8);
        assert_eq!(cmd_estop.command.linear, [0.0, 0.0, 0.0]);
        assert_eq!(cmd_estop.command.angular, [0.0, 0.0, 0.0]);

        // 3. SafeLock / Watchdog aşımı
        let mut tier1_safelock = Tier1SafetyStatus::safe_operating();
        tier1_safelock.safety_mode = SafetyMode::SafeLock;

        let cmd_safelock = arbiter.arbitrate(&tier1_safelock, None, Some(mission_cmd), 0.1, 3000);
        assert_eq!(cmd_safelock.active_tier, PriorityTier::Tier1Safety as u8);
        assert_eq!(
            cmd_safelock.override_reason,
            OverrideReason::WatchdogSafeLock as u8
        );
        assert_eq!(cmd_safelock.command.linear, [0.0, 0.0, 0.0]);
    }

    /// Doğrulama 2: Tier 2 (Engel Kaçınma) anında Tier 3 kullanıcı hedefinin maskelenmesi
    #[test]
    fn test_tier2_safety_override() {
        let mut arbiter = CommandArbiter::default();

        let mission_cmd = KinematicCommand {
            linear: [2.5, 0.0, 0.0], // Görev kodu "Düz git" diyor
            angular: [0.0, 0.0, 0.0],
            timestamp_us: 1000,
        };

        let avoidance_cmd = KinematicCommand {
            linear: [0.8, 0.0, 0.0], // Engel algılandı: yavaşla ve sola kır!
            angular: [0.0, 0.0, 1.5],
            timestamp_us: 1000,
        };

        let tier1 = Tier1SafetyStatus::safe_operating();

        let cmd = arbiter.arbitrate(
            &tier1,
            Some(avoidance_cmd),
            Some(mission_cmd),
            0.1,
            1000,
        );

        assert_eq!(cmd.active_tier, PriorityTier::Tier2Reactive as u8);
        assert_eq!(cmd.override_reason, OverrideReason::Obstacle as u8);
        // Kaçınma komutunun açısal hızı aktarılmış olmalı (> 0)
        assert!(cmd.command.angular[2] > 0.0);
    }

    /// Doğrulama 3: Engel ve emniyet sorunu yokken görev komutunun birebir iletilmesi
    #[test]
    fn test_clear_corridor_passthrough() {
        let mut arbiter = CommandArbiter::new(10.0, 10.0); // Yüksek limit (filtre takılmasın)

        let mission_cmd = KinematicCommand {
            linear: [1.8, 0.0, 0.0],
            angular: [0.0, 0.0, -0.4],
            timestamp_us: 5000,
        };

        let tier1 = Tier1SafetyStatus::safe_operating();

        let cmd = arbiter.arbitrate(&tier1, None, Some(mission_cmd), 0.5, 5000);
        assert_eq!(cmd.active_tier, PriorityTier::Tier3Mission as u8);
        assert_eq!(cmd.override_reason, OverrideReason::None as u8);
        assert!((cmd.command.linear[0] - 1.8).abs() < 1e-4);
        assert!((cmd.command.angular[2] - (-0.4)).abs() < 1e-4);
    }

    /// Doğrulama 4: Hız Değişim Oranı ve Sarsıntı Önleme (SlewRateLimiter)
    #[test]
    fn test_slew_rate_limiter() {
        let mut limiter = SlewRateLimiter::new(2.0, 1.0); // max 2 m/s^2 linear, 1 rad/s^2 angular

        // 0'dan 5 m/s'ye anlık basamak isteği (dt = 0.5s)
        let target = KinematicCommand {
            linear: [5.0, 0.0, 0.0],
            angular: [0.0, 0.0, 2.0],
            timestamp_us: 100,
        };

        // 0.5s sonra: max delta linear = 2.0 * 0.5 = 1.0 m/s
        let filtered = limiter.filter(target, 0.5);
        assert_eq!(filtered.linear[0], 1.0);
        assert_eq!(filtered.angular[2], 0.5);

        // Bir 0.5s sonra: toplam 2.0 m/s olmalı
        let filtered2 = limiter.filter(target, 0.5);
        assert_eq!(filtered2.linear[0], 2.0);
        assert_eq!(filtered2.angular[2], 1.0);
    }

    /// Doğrulama 5: C-ABI 40-Bayt Sıfır Kopyalama ve Hizalama
    #[test]
    fn test_bytemuck_c_abi() {
        let cmd = ArbitratedCommand::new(
            KinematicCommand::new([1.0, 0.0, 0.0], [0.0, 0.0, 0.5], 123456),
            PriorityTier::Tier2Reactive,
            OverrideReason::Obstacle,
        );

        let bytes = cmd.as_bytes();
        assert_eq!(bytes.len(), 40);

        let parsed = ArbitratedCommand::try_from_bytes(bytes).expect("Sıfır kopyalama parse");
        assert_eq!(*parsed, cmd);
    }

    /// Doğrulama 6: SITL Mock Senaryo Simülasyonu (50 Hz Koşu)
    #[test]
    fn test_mock_scenario_50hz() {
        let mut harness = MockArbiterHarness::new();
        let results = harness.run_scenario(100);

        assert_eq!(results.len(), 100);

        // 10. Adım: Temiz koridor (Tier 3)
        assert_eq!(results[10].active_tier, PriorityTier::Tier3Mission as u8);

        // 35. Adım: Engel kaçınma devrede (Tier 2)
        assert_eq!(results[35].active_tier, PriorityTier::Tier2Reactive as u8);
        assert_eq!(results[35].override_reason, OverrideReason::Obstacle as u8);

        // 65. Adım: E-Stop basılı (Tier 1 Veto)
        assert_eq!(results[65].active_tier, PriorityTier::Tier1Safety as u8);
        assert_eq!(results[65].override_reason, OverrideReason::EStop as u8);
        assert_eq!(results[65].command.linear, [0.0, 0.0, 0.0]);

        // 80. Adım: E-Stop kalktı, göreve geri dönüldü (Tier 3)
        assert_eq!(results[80].active_tier, PriorityTier::Tier3Mission as u8);
    }
}
