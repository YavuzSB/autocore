use autocore_base::StateVector6DoF;

/// Kuaterniyon matematiği ve 3D rotasyonlar (w, x, y, z)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self::identity()
    }
}

impl Quat {
    pub fn new(w: f32, x: f32, y: f32, z: f32) -> Self {
        let len = (w * w + x * x + y * y + z * z).sqrt();
        if len > 1e-6 {
            Self {
                w: w / len,
                x: x / len,
                y: y / len,
                z: z / len,
            }
        } else {
            Self::identity()
        }
    }

    pub fn identity() -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    pub fn from_euler(roll_rad: f32, pitch_rad: f32, yaw_rad: f32) -> Self {
        let cr = (roll_rad * 0.5).cos();
        let sr = (roll_rad * 0.5).sin();
        let cp = (pitch_rad * 0.5).cos();
        let sp = (pitch_rad * 0.5).sin();
        let cy = (yaw_rad * 0.5).cos();
        let sy = (yaw_rad * 0.5).sin();

        let w = cr * cp * cy + sr * sp * sy;
        let x = sr * cp * cy - cr * sp * sy;
        let y = cr * sp * cy + sr * cp * sy;
        let z = cr * cp * sy - sr * sp * cy;

        Self::new(w, x, y, z)
    }

    pub fn to_euler(&self) -> (f32, f32, f32) {
        // Roll (x-ekseni dönüşü)
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        // Pitch (y-ekseni dönüşü)
        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            std::f32::consts::FRAC_PI_2.copysign(sinp)
        } else {
            sinp.asin()
        };

        // Yaw (z-ekseni dönüşü)
        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        (roll, pitch, yaw)
    }

    /// İki kuaterniyonun çarpımı ($q_1 \otimes q_2$)
    pub fn multiply(&self, rhs: &Quat) -> Self {
        let w = self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z;
        let x = self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y;
        let y = self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x;
        let z = self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w;
        Self::new(w, x, y, z)
    }

    /// 3B vektörü kuaterniyon rotasyonuyla döndürür ($v' = q \cdot v \cdot q^*$)
    pub fn rotate_vec3(&self, v: [f32; 3]) -> [f32; 3] {
        let q_vec = Quat {
            w: 0.0,
            x: v[0],
            y: v[1],
            z: v[2],
        };
        let q_conj = Quat {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        };
        let res = self.multiply(&q_vec).multiply(&q_conj);
        [res.x, res.y, res.z]
    }
}

/// 6-DoF Durum Kestiricisi (Strapdown Kinematic State Estimator)
pub struct StateEstimator6DoF {
    state: StateVector6DoF,
}

impl Default for StateEstimator6DoF {
    fn default() -> Self {
        Self::new()
    }
}

impl StateEstimator6DoF {
    pub fn new() -> Self {
        let mut state = StateVector6DoF::default();
        state.quaternion = [1.0, 0.0, 0.0, 0.0]; // w, x, y, z
        Self { state }
    }

    pub fn current_state(&self) -> &StateVector6DoF {
        &self.state
    }

    /// Açısal hız (gyro) ve doğrusal ivme (accel) ile durumu zaman adımı $\Delta t$ boyunca entegre eder.
    pub fn integrate(&mut self, angular_vel: [f32; 3], linear_accel: [f32; 3], dt_s: f32, now_us: u64) {
        if dt_s <= 0.0 {
            return;
        }

        let q_curr = Quat::new(
            self.state.quaternion[0],
            self.state.quaternion[1],
            self.state.quaternion[2],
            self.state.quaternion[3],
        );

        // Açısal hız kuaterniyon artışı: dq = 0.5 * q * omega * dt
        let omega_quat = Quat {
            w: 0.0,
            x: angular_vel[0] * dt_s * 0.5,
            y: angular_vel[1] * dt_s * 0.5,
            z: angular_vel[2] * dt_s * 0.5,
        };
        let q_next = q_curr.multiply(&omega_quat);
        let q_updated = Quat::new(
            q_curr.w + q_next.w,
            q_curr.x + q_next.x,
            q_curr.y + q_next.y,
            q_curr.z + q_next.z,
        );

        self.state.quaternion = [q_updated.w, q_updated.x, q_updated.y, q_updated.z];
        self.state.angular_rates = angular_vel;

        // Gövde ivmesini dünya çerçevesine döndür
        let world_accel = q_updated.rotate_vec3(linear_accel);

        // Hız güncellemesi: v = v + a * dt
        self.state.velocity[0] += world_accel[0] * dt_s;
        self.state.velocity[1] += world_accel[1] * dt_s;
        self.state.velocity[2] += world_accel[2] * dt_s;

        // Konum güncellemesi: p = p + v * dt
        self.state.position[0] += self.state.velocity[0] * dt_s;
        self.state.position[1] += self.state.velocity[1] * dt_s;
        self.state.position[2] += self.state.velocity[2] * dt_s;

        self.state.timestamp_us = now_us;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quaternion_euler_roundtrip() {
        let roll = 0.15f32;
        let pitch = -0.25f32;
        let yaw = 1.05f32;

        let q = Quat::from_euler(roll, pitch, yaw);
        let (r, p, y) = q.to_euler();

        assert!((r - roll).abs() < 1e-4);
        assert!((p - pitch).abs() < 1e-4);
        assert!((y - yaw).abs() < 1e-4);
    }

    #[test]
    fn test_state_estimator_integration() {
        let mut est = StateEstimator6DoF::new();

        // 1 m/s^2 x ivmesi ile 1 saniye boyunca ileri git
        for step in 1..=10 {
            est.integrate([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], 0.1, step * 100_000);
        }

        let state = est.current_state();
        assert!((state.velocity[0] - 1.0).abs() < 1e-3);
        // x = 0.5 * a * t^2 = 0.5 metre (Euler yaklaşık 0.55m)
        assert!(state.position[0] > 0.4 && state.position[0] < 0.65);
    }
}
