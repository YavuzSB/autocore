//! autocore-transform :: SE(3) Lie Grubu ve Kuaterniyon Kinematiği

use crate::error::TransformError;
use serde::{Deserialize, Serialize};
use std::ops::Mul;

/// Birim Kuaterniyon [w, x, y, z] (SO(3) Rotasyonu)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self::identity()
    }
}

impl Quaternion {
    /// Birim kuaterniyon (Rotasyon yok)
    pub const fn identity() -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Yeni bir kuaterniyon oluşturur ve normalize eder.
    pub fn new(w: f32, x: f32, y: f32, z: f32) -> Result<Self, TransformError> {
        let norm_sq = w * w + x * x + y * y + z * z;
        if !norm_sq.is_finite() || norm_sq < 1e-12 {
            return Err(TransformError::InvalidQuaternion);
        }
        let inv_norm = 1.0 / norm_sq.sqrt();
        Ok(Self {
            w: w * inv_norm,
            x: x * inv_norm,
            y: y * inv_norm,
            z: z * inv_norm,
        })
    }

    /// Ham [w, x, y, z] dizisinden oluşturur ve normalize eder.
    pub fn from_array(arr: [f32; 4]) -> Result<Self, TransformError> {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }

    /// [w, x, y, z] dizisi olarak döndürür
    pub fn to_array(&self) -> [f32; 4] {
        [self.w, self.x, self.y, self.z]
    }

    /// Kuaterniyon normunu döndürür
    pub fn norm(&self) -> f32 {
        (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Eşlenik kuaterniyon (Conjugate: q* = [w, -x, -y, -z])
    pub fn conjugate(&self) -> Self {
        Self {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    /// Ters kuaterniyon (Birim kuaterniyon için eşleniğe eşittir)
    pub fn inverse(&self) -> Self {
        self.conjugate()
    }

    /// Euler açılarından (Roll, Pitch, Yaw - radyan) ZYX sırasıyla kuaterniyon üretir
    pub fn from_euler(roll: f32, pitch: f32, yaw: f32) -> Self {
        let cr = (roll * 0.5).cos();
        let sr = (roll * 0.5).sin();
        let cp = (pitch * 0.5).cos();
        let sp = (pitch * 0.5).sin();
        let cy = (yaw * 0.5).cos();
        let sy = (yaw * 0.5).sin();

        Self {
            w: cr * cp * cy + sr * sp * sy,
            x: sr * cp * cy - cr * sp * sy,
            y: cr * sp * cy + sr * cp * sy,
            z: cr * cp * sy - sr * sp * cy,
        }
    }

    /// Kuaterniyondan Euler açılarını [roll, pitch, yaw] radyan olarak ayrıştırır
    pub fn to_euler(&self) -> (f32, f32, f32) {
        // Roll (x ekseni etrafında)
        let sinr_cosp = 2.0 * (self.w * self.x + self.y * self.z);
        let cosr_cosp = 1.0 - 2.0 * (self.x * self.x + self.y * self.y);
        let roll = sinr_cosp.atan2(cosr_cosp);

        // Pitch (y ekseni etrafında - Gimbal lock korumalı)
        let sinp = 2.0 * (self.w * self.y - self.z * self.x);
        let pitch = if sinp.abs() >= 1.0 {
            std::f32::consts::FRAC_PI_2.copysign(sinp)
        } else {
            sinp.asin()
        };

        // Yaw (z ekseni etrafında)
        let siny_cosp = 2.0 * (self.w * self.z + self.x * self.y);
        let cosy_cosp = 1.0 - 2.0 * (self.y * self.y + self.z * self.z);
        let yaw = siny_cosp.atan2(cosy_cosp);

        (roll, pitch, yaw)
    }

    /// Verilen 3B vektörü kuaterniyon ile döndürür: v' = q * v * q*
    pub fn rotate_vector(&self, v: [f32; 3]) -> [f32; 3] {
        // Hızlı Rodrigues optimizasyonu: t = 2 * (q_vec x v); v' = v + w*t + (q_vec x t)
        let qv = [self.x, self.y, self.z];
        let t = [
            2.0 * (qv[1] * v[2] - qv[2] * v[1]),
            2.0 * (qv[2] * v[0] - qv[0] * v[2]),
            2.0 * (qv[0] * v[1] - qv[1] * v[0]),
        ];
        let cross_t = [
            qv[1] * t[2] - qv[2] * t[1],
            qv[2] * t[0] - qv[0] * t[2],
            qv[0] * t[1] - qv[1] * t[0],
        ];
        [
            v[0] + self.w * t[0] + cross_t[0],
            v[1] + self.w * t[1] + cross_t[1],
            v[2] + self.w * t[2] + cross_t[2],
        ]
    }

    /// İki kuaterniyon arasında Küresel Doğrusal İnterpolasyon (SLERP)
    pub fn slerp(q0: &Self, q1: &Self, t: f32) -> Self {
        let t_clamped = t.clamp(0.0, 1.0);
        let mut dot = q0.w * q1.w + q0.x * q1.x + q0.y * q1.y + q0.z * q1.z;

        // En kısa küresel yol garantisi
        let q1_target = if dot < 0.0 {
            dot = -dot;
            Self {
                w: -q1.w,
                x: -q1.x,
                y: -q1.y,
                z: -q1.z,
            }
        } else {
            *q1
        };

        // Eğer açılar çok yakınsa doğrusal interpolasyon (sayısal kararlılık)
        if dot > 0.9995 {
            let res = Self {
                w: q0.w + t_clamped * (q1_target.w - q0.w),
                x: q0.x + t_clamped * (q1_target.x - q0.x),
                y: q0.y + t_clamped * (q1_target.y - q0.y),
                z: q0.z + t_clamped * (q1_target.z - q0.z),
            };
            return res.new_normalized_fallback();
        }

        let theta_0 = dot.acos();
        let theta = theta_0 * t_clamped;
        let sin_theta = theta.sin();
        let sin_theta_0 = theta_0.sin();

        let s0 = (theta_0 - theta).cos() - dot * sin_theta / sin_theta_0;
        let s1 = sin_theta / sin_theta_0;

        Self {
            w: s0 * q0.w + s1 * q1_target.w,
            x: s0 * q0.x + s1 * q1_target.x,
            y: s0 * q0.y + s1 * q1_target.y,
            z: s0 * q0.z + s1 * q1_target.z,
        }
    }

    fn new_normalized_fallback(&self) -> Self {
        let norm = self.norm();
        if norm < 1e-12 {
            Self::identity()
        } else {
            let inv = 1.0 / norm;
            Self {
                w: self.w * inv,
                x: self.x * inv,
                y: self.y * inv,
                z: self.z * inv,
            }
        }
    }

    /// 3x3 Rotasyon Matrisine dönüştürür
    pub fn to_rotation_matrix(&self) -> [[f32; 3]; 3] {
        let w = self.w;
        let x = self.x;
        let y = self.y;
        let z = self.z;

        [
            [
                1.0 - 2.0 * (y * y + z * z),
                2.0 * (x * y - w * z),
                2.0 * (x * z + w * y),
            ],
            [
                2.0 * (x * y + w * z),
                1.0 - 2.0 * (x * x + z * z),
                2.0 * (y * z - w * x),
            ],
            [
                2.0 * (x * z - w * y),
                2.0 * (y * z + w * x),
                1.0 - 2.0 * (x * x + y * y),
            ],
        ]
    }
}

impl Mul for Quaternion {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        }
    }
}

/// 3B Rijit Cisim Dönüşümü T ∈ SE(3) (Öteleme ve Kuaterniyon Dönme)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform3D {
    pub translation: [f32; 3],
    pub rotation: Quaternion,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform3D {
    /// Birim dönüşüm (Sıfır öteleme, kimlik rotasyonu)
    pub const fn identity() -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation: Quaternion::identity(),
        }
    }

    /// Yeni öteleme ve kuaterniyon ile oluşturur
    pub fn new(translation: [f32; 3], rotation: Quaternion) -> Self {
        Self {
            translation,
            rotation,
        }
    }

    /// Sadece öteleme içeren dönüşüm
    pub fn from_translation(translation: [f32; 3]) -> Self {
        Self {
            translation,
            rotation: Quaternion::identity(),
        }
    }

    /// Sadece rotasyon içeren dönüşüm
    pub fn from_rotation(rotation: Quaternion) -> Self {
        Self {
            translation: [0.0, 0.0, 0.0],
            rotation,
        }
    }

    /// X, Y, Z ve Euler açıları ile dönüşüm oluşturur
    pub fn from_xyz_rpy(x: f32, y: f32, z: f32, roll: f32, pitch: f32, yaw: f32) -> Self {
        Self {
            translation: [x, y, z],
            rotation: Quaternion::from_euler(roll, pitch, yaw),
        }
    }

    /// İki dönüşümü zincirleme birleştirir: T_{A->C} = T_{A->B} * T_{B->C}
    pub fn compose(&self, next: &Self) -> Self {
        let rotated_trans = self.rotation.rotate_vector(next.translation);
        Self {
            translation: [
                self.translation[0] + rotated_trans[0],
                self.translation[1] + rotated_trans[1],
                self.translation[2] + rotated_trans[2],
            ],
            rotation: self.rotation * next.rotation,
        }
    }

    /// Dönüşümün tersini alır: T^{-1}
    pub fn inverse(&self) -> Self {
        let inv_rot = self.rotation.inverse();
        let inv_trans = inv_rot.rotate_vector([
            -self.translation[0],
            -self.translation[1],
            -self.translation[2],
        ]);
        Self {
            translation: inv_trans,
            rotation: inv_rot,
        }
    }

    /// Bir 3B noktayı dönüştürür: p' = R * p + t
    pub fn transform_point(&self, p: [f32; 3]) -> [f32; 3] {
        let rotated = self.rotation.rotate_vector(p);
        [
            rotated[0] + self.translation[0],
            rotated[1] + self.translation[1],
            rotated[2] + self.translation[2],
        ]
    }

    /// Bir 3B serbest yön vektörünü dönüştürür (öteleme uygulanmaz): v' = R * v
    pub fn transform_vector(&self, v: [f32; 3]) -> [f32; 3] {
        self.rotation.rotate_vector(v)
    }

    /// 4x4 Homojen Dönüşüm Matrisi döndürür
    pub fn to_matrix4x4(&self) -> [[f32; 4]; 4] {
        let r = self.rotation.to_rotation_matrix();
        let t = self.translation;

        [
            [r[0][0], r[0][1], r[0][2], t[0]],
            [r[1][0], r[1][1], r[1][2], t[1]],
            [r[2][0], r[2][1], r[2][2], t[2]],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}
