use std::f64::consts::PI;
use crate::coordinates::NedPoint;

/// L1 Doğrusal Olmayan Yol Takip ve Güdüm Kontrolcüsü (L1 Nonlinear Guidance Logic).
/// Hedef yol çizgisine (Waypoint A -> B) aşırı salınım yapmadan yaklaşmayı sağlar.
#[derive(Debug, Clone)]
pub struct L1GuidanceController {
    pub l1_period_s: f64,
    pub l1_damping: f64,
    pub max_lateral_accel_mps2: f64,
    pub min_ground_speed_mps: f64,
}

impl Default for L1GuidanceController {
    fn default() -> Self {
        Self {
            l1_period_s: 18.0,
            l1_damping: 0.707,
            max_lateral_accel_mps2: 4.0, // Yaklaşık 0.4g limit
            min_ground_speed_mps: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GuidanceCommand {
    pub lateral_accel_mps2: f64,
    pub yaw_rate_cmd_radps: f64,
    pub crosstrack_error_m: f64,
    pub bearing_error_rad: f64,
}

impl L1GuidanceController {
    pub fn new(period_s: f64, damping: f64, max_accel: f64) -> Self {
        assert!(period_s > 0.0 && damping > 0.0 && max_accel > 0.0);
        Self {
            l1_period_s: period_s,
            l1_damping: damping,
            max_lateral_accel_mps2: max_accel,
            min_ground_speed_mps: 1.0,
        }
    }

    /// Mevcut durum ve hedef yol segmentine göre yanal ivme ve yaw hızı komutunu üretir.
    pub fn compute_command(
        &self,
        current_pos: &NedPoint,
        ground_speed: f64,
        heading_rad: f64,
        seg_start: &NedPoint,
        seg_end: &NedPoint,
    ) -> GuidanceCommand {
        let speed = ground_speed.max(self.min_ground_speed_mps);

        // L1 mesafesi: L1 = (1 / pi) * damping * period * V
        let l1_dist = (1.0 / PI) * self.l1_damping * self.l1_period_s * speed;

        // Yol vektörü AB
        let ab_n = seg_end.north_m - seg_start.north_m;
        let ab_e = seg_end.east_m - seg_start.east_m;
        let ab_len = (ab_n * ab_n + ab_e * ab_e).sqrt();

        if ab_len < 1e-4 {
            // Segment boyutu sıfırsa hedefe doğru doğrudan yönel
            let to_target_n = seg_end.north_m - current_pos.north_m;
            let to_target_e = seg_end.east_m - current_pos.east_m;
            let target_bearing = to_target_e.atan2(to_target_n);
            let bearing_error = wrap_pi(target_bearing - heading_rad);

            let a_cmd = (speed * speed / l1_dist) * bearing_error.sin();
            let clamped_a = a_cmd.clamp(-self.max_lateral_accel_mps2, self.max_lateral_accel_mps2);
            return GuidanceCommand {
                lateral_accel_mps2: clamped_a,
                yaw_rate_cmd_radps: clamped_a / speed,
                crosstrack_error_m: 0.0,
                bearing_error_rad: bearing_error,
            };
        }

        let ab_unit_n = ab_n / ab_len;
        let ab_unit_e = ab_e / ab_len;

        // Araçtan A'ya olan vektör
        let ap_n = current_pos.north_m - seg_start.north_m;
        let ap_e = current_pos.east_m - seg_start.east_m;

        // Yola paralel izdüşüm mesafesi
        let along_track = ap_n * ab_unit_n + ap_e * ab_unit_e;

        // Yanal sapma (Crosstrack Error): Pozitif = yolun sağında
        let crosstrack_error = ap_e * ab_unit_n - ap_n * ab_unit_e;

        // Yol üzerinde L1 mesafesindeki referans hedef noktası
        let target_along = along_track + (l1_dist * l1_dist - crosstrack_error * crosstrack_error)
            .max(0.0)
            .sqrt();

        let target_n = seg_start.north_m + target_along * ab_unit_n;
        let target_e = seg_start.east_m + target_along * ab_unit_e;

        // Araçtan L1 referans noktasına olan vektör ve açı
        let to_ref_n = target_n - current_pos.north_m;
        let to_ref_e = target_e - current_pos.east_m;
        let ref_bearing = to_ref_e.atan2(to_ref_n);

        // Hız vektörü ile L1 referans açısı arasındaki fark (eta)
        let eta = wrap_pi(ref_bearing - heading_rad);

        // L1 yanal ivme formülü: a_cmd = 2 * (V^2 / L1) * sin(eta)
        let lateral_accel = 2.0 * (speed * speed / l1_dist) * eta.sin();
        let clamped_accel = lateral_accel.clamp(-self.max_lateral_accel_mps2, self.max_lateral_accel_mps2);

        GuidanceCommand {
            lateral_accel_mps2: clamped_accel,
            yaw_rate_cmd_radps: clamped_accel / speed,
            crosstrack_error_m: crosstrack_error,
            bearing_error_rad: eta,
        }
    }
}

fn wrap_pi(mut angle: f64) -> f64 {
    while angle > PI {
        angle -= 2.0 * PI;
    }
    while angle < -PI {
        angle += 2.0 * PI;
    }
    angle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l1_on_track_zero_accel() {
        let ctrl = L1GuidanceController::default();
        let start = NedPoint::new(0.0, 0.0, 0.0);
        let end = NedPoint::new(1000.0, 0.0, 0.0); // Tam kuzeye giden yol

        // Araç tam yolun üzerinde ve tam kuzeye gidiyor
        let pos = NedPoint::new(200.0, 0.0, 0.0);
        let cmd = ctrl.compute_command(&pos, 20.0, 0.0, &start, &end);

        assert!(cmd.crosstrack_error_m.abs() < 1e-4);
        assert!(cmd.lateral_accel_mps2.abs() < 1e-3);
    }

    #[test]
    fn test_l1_crosstrack_correction() {
        let ctrl = L1GuidanceController::default();
        let start = NedPoint::new(0.0, 0.0, 0.0);
        let end = NedPoint::new(1000.0, 0.0, 0.0);

        // Araç yolun 50 metre doğusunda (sağında)
        let pos = NedPoint::new(200.0, 50.0, 0.0);
        let cmd = ctrl.compute_command(&pos, 20.0, 0.0, &start, &end);

        // Yola geri dönmek için negatif yanal ivme (sola dönüş) üretmeli
        assert!(cmd.crosstrack_error_m > 0.0);
        assert!(cmd.lateral_accel_mps2 < 0.0);
        assert!(cmd.yaw_rate_cmd_radps < 0.0);
    }
}
