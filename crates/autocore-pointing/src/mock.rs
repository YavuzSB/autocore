//! autocore-pointing :: SITL Test İkizi ve Pan-Tilt Servo Atalet Simülatörü

use crate::types::{PointingCommand, TurretFeedback};

/// SITL Test İkizi: 2-DoF Servo motor ataletini (1. Derece Gecikme Filtresi) simüle eder
pub struct MockPanTiltRig {
    pub pan_rad: f32,
    pub tilt_rad: f32,
    pub pan_rate: f32,
    pub tilt_rate: f32,
    /// Servo zaman sabiti (tau ~ 0.04s = 40 ms)
    pub tau_s: f32,
}

impl Default for MockPanTiltRig {
    fn default() -> Self {
        Self::new(0.04)
    }
}

impl MockPanTiltRig {
    pub fn new(tau_s: f32) -> Self {
        Self {
            pan_rad: 0.0,
            tilt_rad: 0.0,
            pan_rate: 0.0,
            tilt_rate: 0.0,
            tau_s: tau_s.max(0.001),
        }
    }

    /// Bir zaman adımı (dt_s) boyunca servoları hedefe doğru sürer
    pub fn step(&mut self, cmd: &PointingCommand, dt_s: f32, timestamp_us: u64) -> TurretFeedback {
        let alpha = 1.0 - (-dt_s / self.tau_s).exp();

        let prev_pan = self.pan_rad;
        let prev_tilt = self.tilt_rad;

        self.pan_rad += (cmd.target_pan_rad - self.pan_rad) * alpha;
        self.tilt_rad += (cmd.target_tilt_rad - self.tilt_rad) * alpha;

        self.pan_rate = (self.pan_rad - prev_pan) / dt_s;
        self.tilt_rate = (self.tilt_rad - prev_tilt) / dt_s;

        TurretFeedback {
            current_pan_rad: self.pan_rad,
            current_tilt_rad: self.tilt_rad,
            pan_rate_rad_s: self.pan_rate,
            tilt_rate_rad_s: self.tilt_rate,
            timestamp_us,
        }
    }
}
