//! autocore-ground :: Rota Takip Kontrolcüleri (Pure Pursuit & Stanley)
//!
//! Zemin araçları için yol eğriliği ve yanal sapmayı açısal hız veya direksiyon açısına çevirir.

use std::f32::consts::PI;

/// Pure Pursuit Rota Takipçisi (Diferansiyel ve Skid-Steer için)
#[derive(Debug, Clone, Copy)]
pub struct PurePursuitTracker {
    /// İleri bakış mesafesi (metre) - tipik 1.0 .. 3.0 m
    pub lookahead_dist_m: f32,
    /// Maksimum açısal hız sınırı (rad/s)
    pub max_angular_rate_rad_s: f32,
}

impl Default for PurePursuitTracker {
    fn default() -> Self {
        Self {
            lookahead_dist_m: 1.5,
            max_angular_rate_rad_s: 2.0,
        }
    }
}

impl PurePursuitTracker {
    pub fn new(lookahead_dist_m: f32, max_angular_rate_rad_s: f32) -> Self {
        Self {
            lookahead_dist_m: lookahead_dist_m.max(0.2),
            max_angular_rate_rad_s: max_angular_rate_rad_s.abs(),
        }
    }

    /// Araç gövdesine göre hedef noktanın [x_ileri, y_sol] ofsetinden
    /// gereken açısal hız komutunu (omega_z) hesaplar:
    /// kappa = 2 * y / (Ld^2)
    /// omega = v * kappa
    pub fn compute_angular_velocity(&self, target_relative_body: [f32; 2], linear_speed: f32) -> f32 {
        let x = target_relative_body[0];
        let y = target_relative_body[1];

        let l_sq = x * x + y * y;
        if l_sq < 1e-4 {
            return 0.0;
        }

        let kappa = (2.0 * y) / l_sq;
        let omega = linear_speed * kappa;
        omega.clamp(-self.max_angular_rate_rad_s, self.max_angular_rate_rad_s)
    }
}

/// Stanley Rota Takip Kontrolcüsü (Ackermann Ön Aks Geometrisi için)
#[derive(Debug, Clone, Copy)]
pub struct StanleyTracker {
    /// Yanal hata kazancı (k)
    pub k_crosstrack: f32,
    /// Düşük hız tekillik önleme sabiti (m/s)
    pub k_soft: f32,
    /// Maksimum direksiyon açısı (radyan)
    pub max_steer_rad: f32,
}

impl Default for StanleyTracker {
    fn default() -> Self {
        Self {
            k_crosstrack: 0.8,
            k_soft: 0.5,
            max_steer_rad: 35.0_f32.to_radians(), // 35 derece
        }
    }
}

impl StanleyTracker {
    pub fn new(k_crosstrack: f32, max_steer_deg: f32) -> Self {
        Self {
            k_crosstrack,
            k_soft: 0.5,
            max_steer_rad: max_steer_deg.to_radians(),
        }
    }

    /// Pruva hatası (heading_error) ve ön aks yanal sapmasından (crosstrack_error)
    /// ön tekerlek direksiyon açısını (delta) hesaplar:
    /// delta = heading_error + atan2(k * e_fa, v + k_soft)
    pub fn compute_steer_angle(
        &self,
        heading_error_rad: f32,
        front_axle_crosstrack_m: f32,
        forward_speed_m_s: f32,
    ) -> f32 {
        let v = forward_speed_m_s.max(0.0);
        let crosstrack_term = (self.k_crosstrack * front_axle_crosstrack_m).atan2(v + self.k_soft);
        let raw_delta = wrap_pi(heading_error_rad) + crosstrack_term;
        raw_delta.clamp(-self.max_steer_rad, self.max_steer_rad)
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
