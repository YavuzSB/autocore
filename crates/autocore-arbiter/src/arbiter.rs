//! autocore-arbiter :: 3 Seviyeli Deterministik Komut Hakemi (CommandArbiter)
//!
//! Tier 1 (Veto) > Tier 2 (Reaktif Engel) > Tier 3 (Kullanıcı Görev Kodu)
//! Mikro saniye seviyesinde deterministik öncelik kararı ve sarsıntısız hız geçişi.

use crate::rate_limiter::SlewRateLimiter;
use crate::types::{ArbitratedCommand, OverrideReason, PriorityTier, Tier1SafetyStatus};
use autocore_base::{KinematicCommand, SafetyMode};

/// Komut Öncelik Hakemi
pub struct CommandArbiter {
    limiter: SlewRateLimiter,
    last_active_tier: PriorityTier,
    last_command: ArbitratedCommand,
}

impl Default for CommandArbiter {
    fn default() -> Self {
        // Varsayılan: 3.0 m/s^2 çizgisel ivme, 4.0 rad/s^2 açısal ivme limiti
        Self::new(3.0, 4.0)
    }
}

impl CommandArbiter {
    pub fn new(max_linear_accel: f32, max_angular_accel: f32) -> Self {
        Self {
            limiter: SlewRateLimiter::new(max_linear_accel, max_angular_accel),
            last_active_tier: PriorityTier::Tier1Safety,
            last_command: ArbitratedCommand::default(),
        }
    }

    /// Son uygulanan hakem çıktısı
    pub fn last_command(&self) -> ArbitratedCommand {
        self.last_command
    }

    /// Son aktif öncelik seviyesi
    pub fn last_active_tier(&self) -> PriorityTier {
        self.last_active_tier
    }

    /// Hakem Karar Algoritması (Mikrosaniyelik Deterministik Değerlendirme)
    pub fn arbitrate(
        &mut self,
        tier1: &Tier1SafetyStatus,
        tier2_reactive: Option<KinematicCommand>,
        tier3_mission: Option<KinematicCommand>,
        dt_s: f32,
        now_us: u64,
    ) -> ArbitratedCommand {
        // -----------------------------------------------------------------
        // [ TIER 1: MUTLAK EMNİYET VE VETO ] - En Yüksek Öncelik
        // -----------------------------------------------------------------
        if tier1.estop_active {
            self.limiter.reset();
            let cmd = ArbitratedCommand::neutral_veto(OverrideReason::EStop, now_us);
            self.update_state(PriorityTier::Tier1Safety, cmd);
            return cmd;
        }

        if tier1.safety_mode == SafetyMode::SafeLock || tier1.watchdog_timeout {
            self.limiter.reset();
            let cmd = ArbitratedCommand::neutral_veto(OverrideReason::WatchdogSafeLock, now_us);
            self.update_state(PriorityTier::Tier1Safety, cmd);
            return cmd;
        }

        if tier1.geofence_breach {
            self.limiter.reset();
            let cmd = ArbitratedCommand::neutral_veto(OverrideReason::GeofenceInhibit, now_us);
            self.update_state(PriorityTier::Tier1Safety, cmd);
            return cmd;
        }

        if tier1.safety_mode == SafetyMode::Disarmed || tier1.safety_mode == SafetyMode::Booting {
            self.limiter.reset();
            let cmd = ArbitratedCommand::neutral_veto(OverrideReason::SystemDisarmed, now_us);
            self.update_state(PriorityTier::Tier1Safety, cmd);
            return cmd;
        }

        // -----------------------------------------------------------------
        // [ TIER 2: REAKTİF GÜVENLİK VE ENGEL KAÇINMA ] - İkinci Öncelik
        // -----------------------------------------------------------------
        if let Some(reactive_cmd) = tier2_reactive {
            let smoothed = self.limiter.filter(reactive_cmd, dt_s);
            let cmd = ArbitratedCommand::new(
                smoothed,
                PriorityTier::Tier2Reactive,
                OverrideReason::Obstacle,
            );
            self.update_state(PriorityTier::Tier2Reactive, cmd);
            return cmd;
        }

        // -----------------------------------------------------------------
        // [ TIER 3: KULLANICI GÖREV KATMANI ] - Üçüncü Öncelik
        // -----------------------------------------------------------------
        if let Some(mission_cmd) = tier3_mission {
            let smoothed = self.limiter.filter(mission_cmd, dt_s);
            let cmd = ArbitratedCommand::new(
                smoothed,
                PriorityTier::Tier3Mission,
                OverrideReason::None,
            );
            self.update_state(PriorityTier::Tier3Mission, cmd);
            return cmd;
        }

        // -----------------------------------------------------------------
        // [ BOŞTA / NÖTR DURUM ]
        // -----------------------------------------------------------------
        let cmd = ArbitratedCommand::new(
            KinematicCommand::neutral(now_us),
            PriorityTier::Tier3Mission,
            OverrideReason::None,
        );
        self.update_state(PriorityTier::Tier3Mission, cmd);
        cmd
    }

    fn update_state(&mut self, tier: PriorityTier, cmd: ArbitratedCommand) {
        self.last_active_tier = tier;
        self.last_command = cmd;
    }
}
