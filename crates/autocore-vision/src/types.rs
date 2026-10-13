use serde::{Deserialize, Serialize};

/// 2B Nesne Sınırlayıcı Kutu (Bounding Box: x, y, width, height)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl BoundingBox {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        assert!(width >= 0.0 && height >= 0.0);
        Self { x, y, width, height }
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Kesişim / Birleşim Oranı (Intersection over Union - IoU)
    pub fn iou(&self, other: &BoundingBox) -> f32 {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        let inter_w = (x2 - x1).max(0.0);
        let inter_h = (y2 - y1).max(0.0);
        let inter_area = inter_w * inter_h;

        let union_area = self.area() + other.area() - inter_area;
        if union_area > 1e-6 {
            inter_area / union_area
        } else {
            0.0
        }
    }
}

/// Yapay zeka modeli çıkarım tespiti
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Detection {
    pub bbox: BoundingBox,
    pub confidence: f32,
    pub class_id: u32,
    pub class_name: String,
}

/// Nesne Takip Durumu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackState {
    Tentative,
    Confirmed,
    Lost,
    Deleted,
}

/// Takip Edilen Hedef (Track)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: u32,
    pub bbox: BoundingBox,
    pub velocity: [f32; 2], // [vx, vy] piksel/kare
    pub state: TrackState,
    pub age: u32,
    pub hits: u32,
    pub time_since_update: u32,
    pub class_id: u32,
}

/// Dost / Düşman Sınıflandırması (Identification Friend or Foe)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetClassification {
    Unknown,
    Friend,
    Foe,
    Neutral,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox_iou() {
        let b1 = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
        let b2 = BoundingBox::new(5.0, 0.0, 10.0, 10.0);

        // Kesişim 5x10 = 50, Birleşim 100 + 100 - 50 = 150 -> IoU = 50/150 = 0.333
        let iou = b1.iou(&b2);
        assert!((iou - 1.0 / 3.0).abs() < 1e-4);

        // Tam örtüşme: IoU = 1.0
        assert_eq!(b1.iou(&b1), 1.0);

        // Ayrık kutular: IoU = 0.0
        let b3 = BoundingBox::new(20.0, 20.0, 5.0, 5.0);
        assert_eq!(b1.iou(&b3), 0.0);
    }
}
