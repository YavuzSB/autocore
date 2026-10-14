//! autocore-arbiter :: SITL Test İkizi ve Mock Hakem Koşucusu

use crate::arbiter::CommandArbiter;
use crate::types::{ArbitratedCommand, PriorityTier, Tier1SafetyStatus};
use autocore_base::KinematicCommand;

/// SITL Test İkizi: 50 Hz döngüde katmanlı komut enjeksiyonu simülatörü
pub struct MockArbiterHarness {
    pub arbiter: CommandArbiter,
}

impl Default for MockArbiterHarness {
    fn default() -> Self {
        Self::new()
    }
}

impl MockArbiterHarness {
    pub fn new() -> Self {
        Self {
            arbiter: CommandArbiter::default(),
        }
    }

    /// N adımlık senaryo simülasyonu çalıştırır (dt = 0.02s / 50 Hz):
    /// - Her adımda Tier 3: 2.0 m/s ileri komut üretilir.
    /// - 30..50 adımları arasında önünde engel belirir (Tier 2 sağa kaçış komutu enjekte edilir).
    /// - 60..70 adımları arasında E-Stop basılır (Tier 1 Veto).
    pub fn run_scenario(&mut self, total_steps: usize) -> Vec<ArbitratedCommand> {
        let dt_s = 0.02; // 50 Hz
        let mut results = Vec::with_capacity(total_steps);

        for step in 0..total_steps {
            let now_us = (step as u64) * 20_000;

            // Tier 1 Durumu
            let estop = (60..=70).contains(&step);
            let tier1 = Tier1SafetyStatus {
                safety_mode: autocore_base::SafetyMode::Mission,
                estop_active: estop,
                geofence_breach: false,
                watchdog_timeout: false,
            };

            // Tier 2 Engel Durumu (30..50 arası)
            let tier2 = if (30..=50).contains(&step) {
                Some(KinematicCommand {
                    linear: [0.5, 0.0, 0.0],
                    angular: [0.0, 0.0, 1.2], // Sağa sert dönüş
                    timestamp_us: now_us,
                })
            } else {
                None
            };

            // Tier 3 Görev Durumu (Sürekli 2.0 m/s ileri)
            let tier3 = Some(KinematicCommand {
                linear: [2.0, 0.0, 0.0],
                angular: [0.0, 0.0, 0.0],
                timestamp_us: now_us,
            });

            let arbitrated = self.arbiter.arbitrate(&tier1, tier2, tier3, dt_s, now_us);
            results.push(arbitrated);
        }

        results
    }

    /// Verilen tek adım için hakem kararı üretir
    pub fn step(
        &mut self,
        tier1: &Tier1SafetyStatus,
        tier2: Option<KinematicCommand>,
        tier3: Option<KinematicCommand>,
        dt_s: f32,
        now_us: u64,
    ) -> ArbitratedCommand {
        self.arbiter.arbitrate(tier1, tier2, tier3, dt_s, now_us)
    }

    pub fn last_active_tier(&self) -> PriorityTier {
        self.arbiter.last_active_tier()
    }
}
