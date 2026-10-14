//! autocore-marine :: Su Üstü Rota Çözücü ve Kavitasyon Önleyici (MarineNavigator)
//!
//! Şamandıra dönüşleri, alabora önleme ve pervanelerin boşa dönmesini (kavitasyon)
//! engelleyen hız profili ve ivme sınırlayıcı.

use crate::types::ThrusterSignals;

/// Kavitasyon ve Alabora Önleyici Seyrüsefer Yöneticisi
#[derive(Debug, Clone)]
pub struct MarineNavigator {
    /// Maksimum itki değişim oranı (% / saniye) - Kavitasyon önleyici slew rate
    pub max_thrust_rate_pct_s: f32,
    /// Maksimum izin verilen yanal ivme (m/s^2) - Alabora önleme limiti (~1.5 m/s^2)
    pub max_lateral_accel_m_s2: f32,
    /// Mevcut iskele ve sağ motor itkileri
    current_port_pct: f32,
    current_stbd_pct: f32,
}

impl Default for MarineNavigator {
    fn default() -> Self {
        Self {
            max_thrust_rate_pct_s: 1.5, // Saniyede en fazla %150 değişim
            max_lateral_accel_m_s2: 1.5,
            current_port_pct: 0.0,
            current_stbd_pct: 0.0,
        }
    }
}

impl MarineNavigator {
    pub fn new(max_thrust_rate_pct_s: f32, max_lateral_accel_m_s2: f32) -> Self {
        Self {
            max_thrust_rate_pct_s: max_thrust_rate_pct_s.max(0.1),
            max_lateral_accel_m_s2: max_lateral_accel_m_s2.max(0.1),
            current_port_pct: 0.0,
            current_stbd_pct: 0.0,
        }
    }

    /// Ani itki değişimlerini kavitasyona girmeyecek şekilde yumuşatır (Slew Rate Limit)
    pub fn filter_cavitation(
        &mut self,
        target_signals: &ThrusterSignals,
        dt_s: f32,
    ) -> ThrusterSignals {
        if dt_s <= 0.0 || !dt_s.is_finite() {
            return *target_signals;
        }

        let max_delta = self.max_thrust_rate_pct_s * dt_s;

        let delta_port = (target_signals.port_pct - self.current_port_pct).clamp(-max_delta, max_delta);
        let delta_stbd = (target_signals.starboard_pct - self.current_stbd_pct).clamp(-max_delta, max_delta);

        self.current_port_pct += delta_port;
        self.current_stbd_pct += delta_stbd;

        ThrusterSignals::new(
            self.current_port_pct,
            self.current_stbd_pct,
            target_signals.rudder_angle_rad,
        )
    }

    /// Hıza göre alabora olmayacak maksimum viraj açısal hızını (rad/s) sınırlar:
    /// omega_max = a_lat_max / V
    pub fn limit_yaw_rate(&self, raw_yaw_rate: f32, speed_m_s: f32) -> f32 {
        let v = speed_m_s.max(0.5);
        let max_omega = self.max_lateral_accel_m_s2 / v;
        raw_yaw_rate.clamp(-max_omega, max_omega)
    }

    pub fn reset(&mut self) {
        self.current_port_pct = 0.0;
        self.current_stbd_pct = 0.0;
    }
}
