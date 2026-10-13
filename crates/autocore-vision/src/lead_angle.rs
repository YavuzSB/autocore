/// Angajman Önleme Geometrisi Çözümü (Lead Solution)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeadSolution {
    pub aim_point: [f32; 3],
    pub time_of_flight_s: f32,
    pub distance_m: f32,
    pub azimuth_rad: f32,
    pub elevation_rad: f32,
}

/// Angajman Önleme Geometrisi Hesaplayıcısı (Lead-Angle Intercept Solver)
/// Hareketli hedefin gelecekteki konumunu mühimmat/lazer hızına göre hesaplar.
#[derive(Debug, Clone)]
pub struct LeadAngleSolver;

impl LeadAngleSolver {
    /// Hareketli hedef için önleme noktasını ve nişan alma açılarını çözer.
    pub fn solve(
        target_pos: [f32; 3],
        target_vel: [f32; 3],
        shooter_pos: [f32; 3],
        projectile_speed_mps: f32,
    ) -> Option<LeadSolution> {
        if projectile_speed_mps <= 1e-3 {
            return None;
        }

        let dx = target_pos[0] - shooter_pos[0];
        let dy = target_pos[1] - shooter_pos[1];
        let dz = target_pos[2] - shooter_pos[2];

        let vx = target_vel[0];
        let vy = target_vel[1];
        let vz = target_vel[2];

        // İkinci derece denklem katsayıları: A*t^2 + B*t + C = 0
        let a = vx * vx + vy * vy + vz * vz - projectile_speed_mps * projectile_speed_mps;
        let b = 2.0 * (dx * vx + dy * vy + dz * vz);
        let c = dx * dx + dy * dy + dz * dz;

        // Hedef zaten namlunun ucundaysa
        if c < 1e-4 {
            return Some(LeadSolution {
                aim_point: target_pos,
                time_of_flight_s: 0.0,
                distance_m: 0.0,
                azimuth_rad: 0.0,
                elevation_rad: 0.0,
            });
        }

        // Kök bulma
        let t = if a.abs() < 1e-6 {
            // Doğrusal denklem durumu (B*t + C = 0)
            if b.abs() < 1e-6 {
                return None;
            }
            let t = -c / b;
            if t > 0.0 {
                t
            } else {
                return None;
            }
        } else {
            let discriminant = b * b - 4.0 * a * c;
            if discriminant < 0.0 {
                return None; // Hedef mermiden daha hızlı uzaklaşıyor, önleme imkânsız
            }

            let sqrt_disc = discriminant.sqrt();
            let t1 = (-b - sqrt_disc) / (2.0 * a);
            let t2 = (-b + sqrt_disc) / (2.0 * a);

            // En küçük pozitif zamanı seç
            match (t1 > 0.0, t2 > 0.0) {
                (true, true) => t1.min(t2),
                (true, false) => t1,
                (false, true) => t2,
                (false, false) => return None,
            }
        };

        // Gelecekteki önleme noktası: P_aim = P_tgt + V_tgt * t
        let aim_x = target_pos[0] + vx * t;
        let aim_y = target_pos[1] + vy * t;
        let aim_z = target_pos[2] + vz * t;

        let aim_dx = aim_x - shooter_pos[0];
        let aim_dy = aim_y - shooter_pos[1];
        let aim_dz = aim_z - shooter_pos[2];

        let distance = (aim_dx * aim_dx + aim_dy * aim_dy + aim_dz * aim_dz).sqrt();
        let horizontal_dist = (aim_dx * aim_dx + aim_dy * aim_dy).sqrt();

        let azimuth = aim_dy.atan2(aim_dx);
        let elevation = aim_dz.atan2(horizontal_dist);

        Some(LeadSolution {
            aim_point: [aim_x, aim_y, aim_z],
            time_of_flight_s: t,
            distance_m: distance,
            azimuth_rad: azimuth,
            elevation_rad: elevation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lead_angle_stationary_target() {
        let tgt = [100.0, 0.0, 0.0];
        let vel = [0.0, 0.0, 0.0];
        let shooter = [0.0, 0.0, 0.0];
        let v_proj = 100.0; // 100 m/s

        let sol = LeadAngleSolver::solve(tgt, vel, shooter, v_proj).unwrap();
        assert!((sol.time_of_flight_s - 1.0).abs() < 1e-4); // 100m / 100m/s = 1s
        assert!((sol.aim_point[0] - 100.0).abs() < 1e-4);
        assert!((sol.azimuth_rad - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_lead_angle_crossing_target() {
        let tgt = [100.0, 0.0, 0.0];
        let vel = [0.0, 20.0, 0.0]; // Yönelim +y (yana geçen hedef) 20 m/s
        let shooter = [0.0, 0.0, 0.0];
        let v_proj = 100.0;

        let sol = LeadAngleSolver::solve(tgt, vel, shooter, v_proj).unwrap();

        // Hedef yana hareket ettiği için nişan noktası y > 0 olmalı
        assert!(sol.aim_point[1] > 0.0);
        assert!(sol.azimuth_rad > 0.0);
        assert!(sol.time_of_flight_s > 1.0); // Düz mesafeden daha uzun sürer
    }
}
