//! autocore-pointing :: 2. Derece Kuadratik Balistik Önleme ve Kurşun Açısı Çözücü
//!
//! Hedef hızı, mesafe, mühimmat namlu çıkış hızı ve yerçekimi düşüşünü
//! hesaba katarak namlunun bakması gereken gelecekteki 3B hedef noktasını çözer.

pub const GRAVITY: f32 = 9.80665; // Standart yerçekimi ivmesi (m/s^2)

/// Balistik Önleme Çözümü Çıktısı
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeadSolution {
    /// Namlunun / Gimbalın yönelmesi gereken 3B nişan noktası (metre)
    pub aim_point: [f32; 3],
    /// Geometrik kesişim noktası (yerçekimi düzeltmesi hariç)
    pub intercept_point: [f32; 3],
    /// Mühimmatın hedefe uçuş süresi (saniye)
    pub time_of_flight_s: f32,
    /// Yerçekimi kaynaklı dikey yükseltme (metre)
    pub gravity_drop_m: f32,
}

pub struct LeadAngleSolver;

impl LeadAngleSolver {
    /// 2. Derece Kuadratik Denklem ile Gelecekteki Kesişim Süresi t ve Nişan Noktasını Çözer:
    /// ||r + v*t||^2 = (v_p * t)^2
    /// (v^2 - v_p^2)*t^2 + 2*(r . v)*t + r^2 = 0
    pub fn solve(
        relative_target_pos: [f32; 3],
        target_velocity: [f32; 3],
        projectile_speed: f32,
    ) -> Option<LeadSolution> {
        if projectile_speed <= 0.0 || !projectile_speed.is_finite() {
            return None;
        }

        let rx = relative_target_pos[0];
        let ry = relative_target_pos[1];
        let rz = relative_target_pos[2];

        let vx = target_velocity[0];
        let vy = target_velocity[1];
        let vz = target_velocity[2];

        let r_sq = rx * rx + ry * ry + rz * rz;
        let v_sq = vx * vx + vy * vy + vz * vz;
        let vp_sq = projectile_speed * projectile_speed;

        let r_dot_v = rx * vx + ry * vy + rz * vz;

        let a = v_sq - vp_sq;
        let b = 2.0 * r_dot_v;
        let c = r_sq;

        // Pozitif en küçük zaman kökünü bul
        let t_opt = if a.abs() < 1e-7 {
            // Birinci derece denklem: b * t + c = 0 => t = -c / b
            if b.abs() > 1e-7 {
                let t = -c / b;
                if t > 0.0 {
                    Some(t)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            let disc = b * b - 4.0 * a * c;
            if disc < 0.0 {
                // Hedef kaçıyor, kesişim imkansız
                None
            } else {
                let sqrt_disc = disc.sqrt();
                let t1 = (-b - sqrt_disc) / (2.0 * a);
                let t2 = (-b + sqrt_disc) / (2.0 * a);

                match (t1 > 0.0, t2 > 0.0) {
                    (true, true) => Some(t1.min(t2)),
                    (true, false) => Some(t1),
                    (false, true) => Some(t2),
                    (false, false) => None,
                }
            }
        };

        let t = t_opt?;

        let intercept_point = [rx + vx * t, ry + vy * t, rz + vz * t];

        // Yerçekimi düşüş telafisi: delta_z = 0.5 * g * t^2
        let gravity_drop_m = 0.5 * GRAVITY * t * t;
        let aim_point = [
            intercept_point[0],
            intercept_point[1],
            intercept_point[2] + gravity_drop_m,
        ];

        Some(LeadSolution {
            aim_point,
            intercept_point,
            time_of_flight_s: t,
            gravity_drop_m,
        })
    }
}
