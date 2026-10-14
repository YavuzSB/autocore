//! autocore-vision :: SITL Test İkizi ve Sentetik Kamera Akışı (SyntheticCameraFeed)
//!
//! Fiziksel kamera olmadan 50 Hz sentetik tespitler ve hareketli hedef kutuları üretir.

use crate::types::{BoundingBox, Detection};

/// SITL Sentetik Kamera Kare Üreticisi
pub struct SyntheticCameraFeed {
    pub frame_width: f32,
    pub frame_height: f32,
    pub step_count: u64,
}

impl Default for SyntheticCameraFeed {
    fn default() -> Self {
        Self::new(1920.0, 1080.0)
    }
}

impl SyntheticCameraFeed {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            frame_width: width,
            frame_height: height,
            step_count: 0,
        }
    }

    /// Bir zaman adımı için sentetik hedef tespitleri üretir
    pub fn generate_frame(&mut self, is_occluded: bool) -> Vec<Detection> {
        self.step_count += 1;

        if is_occluded {
            // Engel arkasına geçti, tespit yok
            return Vec::new();
        }

        let t = (self.step_count as f32) * 0.02; // 50 Hz

        // Hedef 1: Ekran boyunca soldan sağa hareket eden araç (class 1)
        let x1 = 100.0 + 30.0 * t;
        let y1 = 400.0 + 10.0 * (t * 2.0).sin();

        vec![Detection {
            bbox: BoundingBox::new(x1, y1, 80.0, 50.0),
            confidence: 0.92,
            class_id: 1,
            class_name: "target_vehicle".to_string(),
        }]
    }
}
