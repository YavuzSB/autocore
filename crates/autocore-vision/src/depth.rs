//! autocore-vision :: 3B Uzaysal İzdüşüm Motoru (DepthProjector)
//!
//! İğne deliği kamera modeli (Pinhole Camera Model) ve iç kalibrasyon matrisi K ile
//! 2D piksel sınırlayıcı kutuyu kameraya göre 3B metre koordinatına dönüştürür.

use crate::types::BoundingBox;

/// Kamera İç Kalibrasyon Parametreleri (Pinhole Matrisi K)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraIntrinsics {
    /// Odak uzaklığı X (piksel)
    pub fx: f32,
    /// Odak uzaklığı Y (piksel)
    pub fy: f32,
    /// Optik merkez X (ana nokta, piksel)
    pub cx: f32,
    /// Optik merkez Y (ana nokta, piksel)
    pub cy: f32,
    /// Görüntü genişliği (piksel)
    pub image_width: u32,
    /// Görüntü yüksekliği (piksel)
    pub image_height: u32,
}

impl Default for CameraIntrinsics {
    fn default() -> Self {
        // Tipik 1080p kamera (1920x1080, ~70 derece FOV)
        Self {
            fx: 1380.0,
            fy: 1380.0,
            cx: 960.0,
            cy: 540.0,
            image_width: 1920,
            image_height: 1080,
        }
    }
}

/// 3B Uzaysal İzdüşüm Motoru
pub struct DepthProjector {
    pub intrinsics: CameraIntrinsics,
}

impl Default for DepthProjector {
    fn default() -> Self {
        Self::new(CameraIntrinsics::default())
    }
}

impl DepthProjector {
    pub fn new(intrinsics: CameraIntrinsics) -> Self {
        assert!(intrinsics.fx > 0.0 && intrinsics.fy > 0.0);
        Self { intrinsics }
    }

    /// 2D piksel (u, v) ve derinlik Z (metre) bilgisini
    /// kamera optik ekseninde 3B Kartezyen metre vektörüne [X_c, Y_c, Z_c] dönüştürür:
    /// X_c = (u - c_x) * Z / f_x
    /// Y_c = (v - c_y) * Z / f_y
    /// Z_c = Z
    pub fn pixel_to_3d(&self, u: f32, v: f32, depth_m: f32) -> [f32; 3] {
        let z = depth_m.max(0.01);
        let x = (u - self.intrinsics.cx) * z / self.intrinsics.fx;
        let y = (v - self.intrinsics.cy) * z / self.intrinsics.fy;
        [x, y, z]
    }

    /// 2B Bounding Box merkezini ve derinliği 3B uzaysal konuma izdüşürür
    pub fn bbox_to_3d(&self, bbox: &BoundingBox, depth_m: f32) -> [f32; 3] {
        let (u, v) = bbox.center();
        self.pixel_to_3d(u, v, depth_m)
    }

    /// 3B Kartezyen noktayı [X, Y, Z] geri 2D piksel düzlemine izdüşürür:
    /// u = X * f_x / Z + c_x
    /// v = Y * f_y / Z + c_y
    pub fn project_3d_to_pixel(&self, point_3d: [f32; 3]) -> Option<(f32, f32)> {
        let z = point_3d[2];
        if z <= 0.001 {
            return None; // Kameranın arkasında
        }

        let u = point_3d[0] * self.intrinsics.fx / z + self.intrinsics.cx;
        let v = point_3d[1] * self.intrinsics.fy / z + self.intrinsics.cy;

        Some((u, v))
    }
}
