//! autocore-arbiter :: Hız Değişim Oranı Sınırlayıcı (SlewRateLimiter)
//!
//! Seviyeler arası geçişlerde ve sert hız taleplerinde mekanik sarsıntıyı (jerk)
//! önleyen ve motorları koruyan ivme filtresi.

use autocore_base::KinematicCommand;

/// Çizgisel ve Açısal İvme Sınırlayıcı
#[derive(Debug, Clone)]
pub struct SlewRateLimiter {
    /// Maksimum çizgisel ivme (m/s^2)
    pub max_linear_accel: f32,
    /// Maksimum açısal ivme (rad/s^2)
    pub max_angular_accel: f32,
    /// Önceki uygulanan çizgisel hız [vx, vy, vz]
    current_linear: [f32; 3],
    /// Önceki uygulanan açısal hız [wx, wy, wz]
    current_angular: [f32; 3],
}

impl SlewRateLimiter {
    pub fn new(max_linear_accel: f32, max_angular_accel: f32) -> Self {
        Self {
            max_linear_accel: max_linear_accel.abs(),
            max_angular_accel: max_angular_accel.abs(),
            current_linear: [0.0; 3],
            current_angular: [0.0; 3],
        }
    }

    /// Filtre durumunu anında sıfırlar (Tier 1 E-Stop durumunda çağrılır)
    pub fn reset(&mut self) {
        self.current_linear = [0.0; 3];
        self.current_angular = [0.0; 3];
    }

    /// Verilen komutu ivme limitlerine göre filtreleyerek yumuşatır
    pub fn filter(&mut self, target: KinematicCommand, dt_s: f32) -> KinematicCommand {
        if dt_s <= 0.0 || !dt_s.is_finite() {
            return target;
        }

        let max_d_lin = self.max_linear_accel * dt_s;
        let max_d_ang = self.max_angular_accel * dt_s;

        let mut filtered_linear = [0.0; 3];
        for i in 0..3 {
            let delta = target.linear[i] - self.current_linear[i];
            let clamped_delta = delta.clamp(-max_d_lin, max_d_lin);
            self.current_linear[i] += clamped_delta;
            filtered_linear[i] = self.current_linear[i];
        }

        let mut filtered_angular = [0.0; 3];
        for i in 0..3 {
            let delta = target.angular[i] - self.current_angular[i];
            let clamped_delta = delta.clamp(-max_d_ang, max_d_ang);
            self.current_angular[i] += clamped_delta;
            filtered_angular[i] = self.current_angular[i];
        }

        KinematicCommand {
            linear: filtered_linear,
            angular: filtered_angular,
            timestamp_us: target.timestamp_us,
        }
    }

    pub fn current_linear(&self) -> [f32; 3] {
        self.current_linear
    }

    pub fn current_angular(&self) -> [f32; 3] {
        self.current_angular
    }
}
