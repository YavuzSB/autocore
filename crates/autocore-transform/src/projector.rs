//! autocore-transform :: Kamera Optik ve Robotik Mekanik Eksen İzdüşüm Motoru
//!
//! Bilgisayarlı görü standartları: Z ileri, X sağ, Y aşağı
//! Mekanik gövde (ISO 8855 / ROS): X ileri, Y sol, Z yukarı
//! Havacılık gövde (NED): X ileri, Y sağ, Z aşağı

use crate::se3::{Quaternion, Transform3D};

/// Optik ve Mekanik Eksen Dönüştürücü (FrameProjector)
pub struct FrameProjector;

impl FrameProjector {
    /// Kamera optik çerçevesindeki [X_sağ, Y_aşağı, Z_ileri] metre vektörünü
    /// mekanik robot gövdesine [X_ileri, Y_sol, Z_yukarı] sıfır kopyalama ile çevirir.
    #[inline(always)]
    pub fn optical_to_body(p_optical: [f32; 3]) -> [f32; 3] {
        [p_optical[2], -p_optical[0], -p_optical[1]]
    }

    /// Mekanik robot gövdesindeki [X_ileri, Y_sol, Z_yukarı] metre vektörünü
    /// kamera optik çerçevesine [X_sağ, Y_aşağı, Z_ileri] çevirir.
    #[inline(always)]
    pub fn body_to_optical(p_body: [f32; 3]) -> [f32; 3] {
        [-p_body[1], -p_body[2], p_body[0]]
    }

    /// Kamera optik çerçevesindeki [X_sağ, Y_aşağı, Z_ileri] metre vektörünü
    /// havacılık gövdesine [X_ileri, Y_sağ, Z_aşağı] (NED Body) çevirir.
    #[inline(always)]
    pub fn optical_to_body_ned(p_optical: [f32; 3]) -> [f32; 3] {
        [p_optical[2], p_optical[0], p_optical[1]]
    }

    /// Havacılık gövdesindeki [X_ileri, Y_sağ, Z_aşağı] metre vektörünü
    /// kamera optik çerçevesine [X_sağ, Y_aşağı, Z_ileri] çevirir.
    #[inline(always)]
    pub fn body_ned_to_optical(p_body_ned: [f32; 3]) -> [f32; 3] {
        [p_body_ned[1], p_body_ned[2], p_body_ned[0]]
    }

    /// Kamera optik çerçevesinden mekanik gövdeye dönüştüren SE(3) Transform nesnesi
    pub fn optical_to_body_transform() -> Transform3D {
        // q = [0.5, -0.5, 0.5, -0.5]
        let quat = Quaternion {
            w: 0.5,
            x: -0.5,
            y: 0.5,
            z: -0.5,
        };
        Transform3D::from_rotation(quat)
    }
}
