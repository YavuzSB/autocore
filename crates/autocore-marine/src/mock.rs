//! autocore-marine :: SITL Test İkizi ve Mock Deniz Simülatörü (MockMarineSimulator)

use crate::types::ThrusterSignals;

/// SITL Mock İDA Simülatörü: Yanal akıntı ve dalga sürüklenmesini simüle eder
pub struct MockMarineSimulator {
    pub x_m: f32,
    pub y_m: f32,
    pub heading_rad: f32,
    pub forward_speed_m_s: f32,
    /// Yanal su akıntısı hızı (m/s)
    pub ocean_current_lateral_m_s: f32,
}

impl Default for MockMarineSimulator {
    fn default() -> Self {
        Self::new(1.0) // 1.0 m/s yanal akıntı
    }
}

impl MockMarineSimulator {
    pub fn new(ocean_current_lateral_m_s: f32) -> Self {
        Self {
            x_m: 0.0,
            y_m: 0.0,
            heading_rad: 0.0,
            forward_speed_m_s: 0.0,
            ocean_current_lateral_m_s,
        }
    }

    /// Bir zaman adımı simülasyon işletir (dt_s = 0.05s / 20 Hz)
    pub fn step(&mut self, signals: &ThrusterSignals, dt_s: f32) {
        // İskele ve sağ motor itkisinden ileri ivme ve dönme torku
        let avg_thrust = (signals.port_pct + signals.starboard_pct) * 0.5;
        let diff_thrust = signals.starboard_pct - signals.port_pct;

        // İleri hız (direnç katsayısı ile dengelenmiş)
        self.forward_speed_m_s += (avg_thrust * 4.0 - self.forward_speed_m_s * 0.5) * dt_s;

        // Pruva açısı değişimi (yaw rate)
        let yaw_rate = diff_thrust * 1.5;
        self.heading_rad += yaw_rate * dt_s;

        // Kinematik yer değiştirme: Motor hızı + Akıntı hızı
        let vx = self.forward_speed_m_s * self.heading_rad.cos();
        let vy = self.forward_speed_m_s * self.heading_rad.sin() + self.ocean_current_lateral_m_s;

        self.x_m += vx * dt_s;
        self.y_m += vy * dt_s;
    }
}
