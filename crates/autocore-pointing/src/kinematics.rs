//! autocore-pointing :: 2-DoF Ters Kinematik ve Eklem Limitleri (Pan/Tilt)
//!
//! Kartezyen hedef konumundan Azimut (Pan) ve Elevasyon (Tilt) açı/hız dönüşümü.

use std::f32::consts::PI;

/// 2-DoF Pan/Tilt Eklem Limitleri ve Ters Kinematik Motoru
#[derive(Debug, Clone, Copy)]
pub struct InverseKinematics2DoF {
    pub min_pan_rad: f32,
    pub max_pan_rad: f32,
    pub min_tilt_rad: f32,
    pub max_tilt_rad: f32,
}

impl Default for InverseKinematics2DoF {
    fn default() -> Self {
        Self {
            min_pan_rad: -PI,
            max_pan_rad: PI,
            min_tilt_rad: -15.0_f32.to_radians(), // -15 derece
            max_tilt_rad: 85.0_f32.to_radians(),  // +85 derece
        }
    }
}

/// 2-DoF Kinematik Çözüm Çıktısı
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JointSetpoints {
    pub pan_rad: f32,
    pub tilt_rad: f32,
    pub pan_rate_rad_s: f32,
    pub tilt_rate_rad_s: f32,
    pub within_limits: bool,
}

impl InverseKinematics2DoF {
    pub fn new(
        min_pan_rad: f32,
        max_pan_rad: f32,
        min_tilt_rad: f32,
        max_tilt_rad: f32,
    ) -> Self {
        Self {
            min_pan_rad,
            max_pan_rad,
            min_tilt_rad,
            max_tilt_rad,
        }
    }

    /// 3B Kartezyen hedef vektöründen ve hedef hızından Pan/Tilt açı ve hızlarını çözer
    pub fn solve(&self, aim_point: [f32; 3], target_velocity: [f32; 3]) -> JointSetpoints {
        let x = aim_point[0];
        let y = aim_point[1];
        let z = aim_point[2];

        let r_xy_sq = x * x + y * y;
        let r_xy = r_xy_sq.sqrt();
        let r_xyz_sq = r_xy_sq + z * z;

        // Azimut (Pan): atan2(y, x)
        let raw_pan = y.atan2(x);
        // Elevasyon (Tilt): atan2(z, r_xy)
        let raw_tilt = z.atan2(r_xy);

        // Açısal hızlar (Jakobiyen türevler)
        let (pan_rate, tilt_rate) = if r_xy > 1e-4 && r_xyz_sq > 1e-4 {
            let vx = target_velocity[0];
            let vy = target_velocity[1];
            let vz = target_velocity[2];

            let p_rate = (x * vy - y * vx) / r_xy_sq;
            let radial_xy_vel = (x * vx + y * vy) / r_xy;
            let t_rate = (r_xy * vz - z * radial_xy_vel) / r_xyz_sq;
            (p_rate, t_rate)
        } else {
            (0.0, 0.0)
        };

        // Eklem limitleri kontrolü ve kelepçeleme
        let pan_clamped = raw_pan.clamp(self.min_pan_rad, self.max_pan_rad);
        let tilt_clamped = raw_tilt.clamp(self.min_tilt_rad, self.max_tilt_rad);

        let within_limits = (raw_pan - pan_clamped).abs() < 1e-4
            && (raw_tilt - tilt_clamped).abs() < 1e-4;

        JointSetpoints {
            pan_rad: pan_clamped,
            tilt_rad: tilt_clamped,
            pan_rate_rad_s: pan_rate,
            tilt_rate_rad_s: tilt_rate,
            within_limits,
        }
    }
}
