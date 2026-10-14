//! autocore-aerial :: Park et al. L1 Doğrusal Olmayan Yol Güdümü (L1Guidance)
//!
//! Sabit ve döner kanat İHA'ların iki yol noktası (waypoint) arasındaki doğruyu
//! rüzgar sapmasını kompanse ederek takip etmesini sağlayan yanal ivme kontrolcüsü.

use std::f32::consts::PI;

/// L1 Güdüm Çıktısı
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct L1Output {
    /// Komuta edilen yanal ivme (m/s^2, pozitif = sağa dönüş)
    pub lateral_accel_m_s2: f32,
    /// Komuta edilen yaw açısal hızı (rad/s)
    pub yaw_rate_rad_s: f32,
    /// Yoldan yanal sapma mesafesi (metre)
    pub crosstrack_error_m: f32,
    /// L1 referans mesafesi (metre)
    pub l1_distance_m: f32,
}

/// L1 Doğrusal Olmayan Güdüm Kontrolcüsü
#[derive(Debug, Clone, Copy)]
pub struct L1Guidance {
    /// L1 doğal periyodu (saniye) - tipik 10-20s
    pub l1_period_s: f32,
    /// L1 sönüm oranı (damping ratio) - tipik 0.707
    pub l1_damping: f32,
    /// Maksimum yanal ivme sınırı (m/s^2) - tipik 4.0 (~0.4g)
    pub max_lateral_accel_m_s2: f32,
    /// Minimum yer hızı eşiği (m/s)
    pub min_ground_speed_m_s: f32,
}

impl Default for L1Guidance {
    fn default() -> Self {
        Self {
            l1_period_s: 15.0,
            l1_damping: 0.707,
            max_lateral_accel_m_s2: 4.0,
            min_ground_speed_m_s: 2.0,
        }
    }
}

impl L1Guidance {
    pub fn new(l1_period_s: f32, l1_damping: f32, max_lateral_accel_m_s2: f32) -> Self {
        Self {
            l1_period_s: l1_period_s.max(1.0),
            l1_damping: l1_damping.max(0.1),
            max_lateral_accel_m_s2: max_lateral_accel_m_s2.abs(),
            min_ground_speed_m_s: 2.0,
        }
    }

    /// Verilen konum, hız ve yol çizgisine (seg_start -> seg_end) göre L1 komutunu hesaplar
    pub fn compute(
        &self,
        current_pos_ned: [f32; 2], // [North, East]
        ground_speed: f32,
        heading_rad: f32,
        seg_start_ned: [f32; 2],
        seg_end_ned: [f32; 2],
    ) -> L1Output {
        let speed = ground_speed.max(self.min_ground_speed_m_s);

        // L1 Mesafesi: L1 = (1 / pi) * damping * period * V
        let l1_dist = (1.0 / PI) * self.l1_damping * self.l1_period_s * speed;

        // Yol vektörü AB
        let ab_n = seg_end_ned[0] - seg_start_ned[0];
        let ab_e = seg_end_ned[1] - seg_start_ned[1];
        let ab_len = (ab_n * ab_n + ab_e * ab_e).sqrt();

        if ab_len < 1e-4 {
            // Segment boyutu sıfırsa doğrudan noktaya yönel
            let to_tgt_n = seg_end_ned[0] - current_pos_ned[0];
            let to_tgt_e = seg_end_ned[1] - current_pos_ned[1];
            let tgt_bearing = to_tgt_e.atan2(to_tgt_n);
            let eta = wrap_pi(tgt_bearing - heading_rad);

            let raw_accel = 2.0 * (speed * speed / l1_dist) * eta.sin();
            let clamped_accel = raw_accel.clamp(
                -self.max_lateral_accel_m_s2,
                self.max_lateral_accel_m_s2,
            );
            return L1Output {
                lateral_accel_m_s2: clamped_accel,
                yaw_rate_rad_s: clamped_accel / speed,
                crosstrack_error_m: 0.0,
                l1_distance_m: l1_dist,
            };
        }

        // Yol birim vektörü
        let u_n = ab_n / ab_len;
        let u_e = ab_e / ab_len;

        // Araçtan yol başlangıcına vektör (AP)
        let ap_n = current_pos_ned[0] - seg_start_ned[0];
        let ap_e = current_pos_ned[1] - seg_start_ned[1];

        // İzdüşüm mesafesi (along-track)
        let along_track = ap_n * u_n + ap_e * u_e;

        // Yanal sapma (crosstrack error: pozitif = yolun sağında)
        let crosstrack_error = ap_e * u_n - ap_n * u_e;

        // L1 referans noktasının yol üzerindeki izdüşümü
        let l1_along = along_track + (l1_dist * l1_dist - crosstrack_error * crosstrack_error).max(0.0).sqrt();

        // L1 hedef noktası
        let ref_pt_n = seg_start_ned[0] + u_n * l1_along;
        let ref_pt_e = seg_start_ned[1] + u_e * l1_along;

        // Araçtan referans noktasına kerteriz
        let to_ref_n = ref_pt_n - current_pos_ned[0];
        let to_ref_e = ref_pt_e - current_pos_ned[1];
        let ref_bearing = to_ref_e.atan2(to_ref_n);

        // Açı farkı eta (heading ile L1 vektörü arası)
        let eta = wrap_pi(ref_bearing - heading_rad);

        // Park et al. formülü: a_s = 2 * (V^2 / L1) * sin(eta)
        let raw_accel = 2.0 * (speed * speed / l1_dist) * eta.sin();
        let clamped_accel = raw_accel.clamp(
            -self.max_lateral_accel_m_s2,
            self.max_lateral_accel_m_s2,
        );

        L1Output {
            lateral_accel_m_s2: clamped_accel,
            yaw_rate_rad_s: clamped_accel / speed,
            crosstrack_error_m: crosstrack_error,
            l1_distance_m: l1_dist,
        }
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
