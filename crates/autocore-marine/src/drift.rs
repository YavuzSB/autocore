//! autocore-marine :: Akıntı ve Dalga Sürüklenme Kompanzatörü (DriftCompensator)
//!
//! Deniz akıntısı ve rüzgar kaynaklı yanal kaymayı integral süzgeciyle
//! toplayarak pruva yengeç açısına (Crab Angle) çevirir ve hedef çizgide tutar.

use std::f32::consts::PI;

/// Akıntı ve Yengeç Açısı (Crab Angle) Kompanzatörü
#[derive(Debug, Clone)]
pub struct DriftCompensator {
    /// İntegral kazancı (ki)
    pub ki: f32,
    /// Maksimum izin verilen yengeç açısı (radyan) - tipik 30 derece (~0.52 rad)
    pub max_crab_angle_rad: f32,
    /// Birikimli yanal hata integrali
    integral_ey: f32,
    /// Anti-windup integral sınırı
    integral_limit: f32,
}

impl Default for DriftCompensator {
    fn default() -> Self {
        Self {
            ki: 0.05,
            max_crab_angle_rad: 30.0_f32.to_radians(), // 30 derece
            integral_ey: 0.0,
            integral_limit: 10.0,
        }
    }
}

impl DriftCompensator {
    pub fn new(ki: f32, max_crab_angle_deg: f32) -> Self {
        Self {
            ki,
            max_crab_angle_rad: max_crab_angle_deg.to_radians(),
            integral_ey: 0.0,
            integral_limit: 10.0,
        }
    }

    /// Yanal mesafe hatası (ey), tahmin edilen akıntı hızı ve ileri hızdan
    /// kompanse edilmiş hedef yönelim açısını (heading_cmd) üretir:
    /// beta = atan2(v_drift + ki * int(ey), v_forward)
    /// heading_cmd = path_heading - beta
    pub fn compute_compensated_heading(
        &mut self,
        path_heading_rad: f32,
        crosstrack_error_m: f32,
        estimated_drift_m_s: f32,
        forward_speed_m_s: f32,
        dt_s: f32,
    ) -> (f32, f32) {
        // İntegral güncellemesi (anti-windup korumalı)
        self.integral_ey += crosstrack_error_m * dt_s;
        self.integral_ey = self.integral_ey.clamp(-self.integral_limit, self.integral_limit);

        let v_fwd = forward_speed_m_s.max(0.5); // Sıfıra bölme koruması
        let total_drift_correction = estimated_drift_m_s + self.ki * self.integral_ey;

        // Yengeç açısı (Crab Angle)
        let raw_crab = total_drift_correction.atan2(v_fwd);
        let crab_angle = raw_crab.clamp(-self.max_crab_angle_rad, self.max_crab_angle_rad);

        // Kompanse edilmiş hedef yönelim
        let compensated_heading = wrap_pi(path_heading_rad - crab_angle);

        (compensated_heading, crab_angle)
    }

    pub fn reset_integral(&mut self) {
        self.integral_ey = 0.0;
    }
}

fn wrap_pi(mut angle: f32) -> f32 {
    while angle > PI {
        angle -= 2.0 * PI;
    }
    while angle < -PI {
        angle += 2.0 * PI;
    }
    angle
}
