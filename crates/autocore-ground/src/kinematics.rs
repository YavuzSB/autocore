/// Diferansiyel Sürüş Kinematiği (İKA / Çift Tekerlekli Robot)
#[derive(Debug, Clone)]
pub struct DifferentialDriveKinematics {
    pub wheelbase_m: f64,
    pub wheel_radius_m: f64,
    pub max_wheel_speed_radps: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelSpeeds {
    pub left_radps: f64,
    pub right_radps: f64,
}

impl DifferentialDriveKinematics {
    pub fn new(wheelbase_m: f64, wheel_radius_m: f64, max_wheel_speed_radps: f64) -> Self {
        assert!(wheelbase_m > 0.0 && wheel_radius_m > 0.0 && max_wheel_speed_radps > 0.0);
        Self {
            wheelbase_m,
            wheel_radius_m,
            max_wheel_speed_radps,
        }
    }

    /// İleri kinematik: Tekerlek açısal hızlarından robotun doğrusal ve açısal hızını hesaplar.
    pub fn forward(&self, wheels: &WheelSpeeds) -> (f64, f64) {
        let v_left = wheels.left_radps * self.wheel_radius_m;
        let v_right = wheels.right_radps * self.wheel_radius_m;

        let v = (v_right + v_left) * 0.5;
        let omega = (v_right - v_left) / self.wheelbase_m;

        (v, omega)
    }

    /// Ters kinematik: İstenen doğrusal hız ($v$) ve açısal hızdan ($\omega$) tekerlek hızlarını hesaplar.
    /// Doyum durumunda eğrilik ($v/\omega$) bozulmayacak şekilde hızlar orantılı olarak ölçeklenir.
    pub fn inverse(&self, v: f64, omega: f64) -> WheelSpeeds {
        let half_base = self.wheelbase_m * 0.5;

        let v_right = v + omega * half_base;
        let v_left = v - omega * half_base;

        let mut right_radps = v_right / self.wheel_radius_m;
        let mut left_radps = v_left / self.wheel_radius_m;

        // Doyum kontrolü ve orantısal ölçekleme (eğriliği koru)
        let max_req = right_radps.abs().max(left_radps.abs());
        if max_req > self.max_wheel_speed_radps {
            let scale = self.max_wheel_speed_radps / max_req;
            right_radps *= scale;
            left_radps *= scale;
        }

        WheelSpeeds {
            left_radps,
            right_radps,
        }
    }
}

/// Ackerman Direksiyon Kinematiği
#[derive(Debug, Clone)]
pub struct AckermannKinematics {
    pub wheelbase_m: f64,
    pub track_width_m: f64,
    pub max_steer_angle_rad: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AckermannCommand {
    pub steer_angle_rad: f64,
    pub inner_angle_rad: f64,
    pub outer_angle_rad: f64,
    pub velocity_mps: f64,
}

impl AckermannKinematics {
    pub fn new(wheelbase_m: f64, track_width_m: f64, max_steer_angle_rad: f64) -> Self {
        assert!(wheelbase_m > 0.0 && track_width_m > 0.0 && max_steer_angle_rad > 0.0);
        Self {
            wheelbase_m,
            track_width_m,
            max_steer_angle_rad,
        }
    }

    /// $(v, \omega)$ hız talebinden Ackerman direksiyon açılarını türetir.
    pub fn from_twist(&self, v: f64, omega: f64) -> AckermannCommand {
        if omega.abs() < 1e-4 || v.abs() < 1e-4 {
            return AckermannCommand {
                steer_angle_rad: 0.0,
                inner_angle_rad: 0.0,
                outer_angle_rad: 0.0,
                velocity_mps: v,
            };
        }

        let turning_radius = v / omega;
        let center_steer = (self.wheelbase_m / turning_radius.abs())
            .atan()
            .copysign(turning_radius);
        let clamped_steer = center_steer.clamp(-self.max_steer_angle_rad, self.max_steer_angle_rad);

        // İç ve dış tekerlek Ackerman geometrisi
        let r_abs = self.wheelbase_m / clamped_steer.abs().tan();
        let half_w = self.track_width_m * 0.5;

        let inner = (self.wheelbase_m / (r_abs - half_w)).atan().copysign(clamped_steer);
        let outer = (self.wheelbase_m / (r_abs + half_w)).atan().copysign(clamped_steer);

        AckermannCommand {
            steer_angle_rad: clamped_steer,
            inner_angle_rad: inner,
            outer_angle_rad: outer,
            velocity_mps: v,
        }
    }
}

/// İDA (İnsansız Deniz Aracı) Çift İtki Güç Dağıtım Matrisi
#[derive(Debug, Clone)]
pub struct IdaThrusterMatrix {
    pub thruster_separation_m: f64,
    pub max_thrust_n: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThrusterAllocation {
    pub left_n: f64,
    pub right_n: f64,
}

impl IdaThrusterMatrix {
    pub fn new(thruster_separation_m: f64, max_thrust_n: f64) -> Self {
        assert!(thruster_separation_m > 0.0 && max_thrust_n > 0.0);
        Self {
            thruster_separation_m,
            max_thrust_n,
        }
    }

    /// İleri itki ($F_x$) ve dönüş torku ($M_z$) talebinden sağ/sol motor itkilerini hesaplar.
    pub fn allocate(&self, surge_force_n: f64, yaw_moment_nm: f64) -> ThrusterAllocation {
        let diff_thrust = yaw_moment_nm / self.thruster_separation_m;

        let mut left = surge_force_n * 0.5 - diff_thrust;
        let mut right = surge_force_n * 0.5 + diff_thrust;

        let max_req = left.abs().max(right.abs());
        if max_req > self.max_thrust_n {
            let scale = self.max_thrust_n / max_req;
            left *= scale;
            right *= scale;
        }

        ThrusterAllocation {
            left_n: left,
            right_n: right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_drive_roundtrip() {
        let diff = DifferentialDriveKinematics::new(0.5, 0.1, 50.0);

        let v = 1.0; // 1 m/s
        let omega = 0.5; // 0.5 rad/s

        let wheels = diff.inverse(v, omega);
        let (rec_v, rec_omega) = diff.forward(&wheels);

        assert!((rec_v - v).abs() < 1e-4);
        assert!((rec_omega - omega).abs() < 1e-4);
    }

    #[test]
    fn test_ackermann_kinematics() {
        let ack = AckermannKinematics::new(1.0, 0.6, 0.5);
        let cmd = ack.from_twist(2.0, 1.0);

        assert!(cmd.steer_angle_rad > 0.0);
        assert!(cmd.inner_angle_rad > cmd.outer_angle_rad); // İç tekerlek daha dik dönmeli
    }

    #[test]
    fn test_ida_thruster_allocation() {
        let ida = IdaThrusterMatrix::new(1.0, 100.0);

        // Tam ileri itki, sıfır dönüş
        let alloc1 = ida.allocate(50.0, 0.0);
        assert_eq!(alloc1.left_n, 25.0);
        assert_eq!(alloc1.right_n, 25.0);

        // Sağa dönüş torku (sağ motor daha az, sol motor daha çok)
        let alloc2 = ida.allocate(50.0, 10.0);
        assert!(alloc2.right_n > alloc2.left_n);
    }
}
