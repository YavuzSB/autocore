//! autocore-safety :: Çift Kademeli Geofence ve Dinamik Sınır Kelepçelemesi (Clamping Engine)

use autocore_base::KinematicCommand;
use geo::{Contains, Point, Polygon};
use tracing::warn;

/// Dinamik Sınır ve Geofence Denetim Motoru
#[derive(Debug, Clone)]
pub struct ClampingEngine {
    /// 2D Yatay Geofence Çokgeni (İzin Verilen Saha)
    boundary_polygon: Option<Polygon<f64>>,
    /// İrtifa / Z ekseni sınırları (Min Z, Max Z)
    z_bounds: (f64, f64),
    /// İzin verilen azami doğrusal bileşen hızı (m/s)
    max_linear_speed: f32,
    /// İzin verilen azami açısal bileşen hızı (rad/s)
    max_angular_speed: f32,
    /// Eylem / Tetikleme Yasak Alanları (Inhibit Zones)
    inhibit_zones: Vec<Polygon<f64>>,
}

impl Default for ClampingEngine {
    fn default() -> Self {
        Self {
            boundary_polygon: None,
            z_bounds: (-1000.0, 1000.0),
            max_linear_speed: 15.0, // 15 m/s (~54 km/h)
            max_angular_speed: 3.14159, // ~180 deg/s
            inhibit_zones: Vec::new(),
        }
    }
}

impl ClampingEngine {
    pub fn new(
        boundary_polygon: Option<Polygon<f64>>,
        z_bounds: (f64, f64),
        max_linear_speed: f32,
        max_angular_speed: f32,
    ) -> Self {
        Self {
            boundary_polygon,
            z_bounds,
            max_linear_speed,
            max_angular_speed,
            inhibit_zones: Vec::new(),
        }
    }

    /// Yeni bir eylem yasak alanı (Inhibit Zone) ekler
    pub fn add_inhibit_zone(&mut self, polygon: Polygon<f64>) {
        self.inhibit_zones.push(polygon);
    }

    /// Kinematik hız komutunu mekanik sınırlara kırpar (Software Clamping)
    /// Eğer herhangi bir değer kırpıldıysa `true` döner.
    pub fn clamp_command(&self, cmd: &mut KinematicCommand) -> bool {
        let mut clamped = false;

        // Doğrusal hız bileşenlerini kırp ve NaN/Inf zehirlenmesine karşı koru
        for val in &mut cmd.linear {
            if !val.is_finite() {
                *val = 0.0;
                clamped = true;
            } else if *val > self.max_linear_speed {
                *val = self.max_linear_speed;
                clamped = true;
            } else if *val < -self.max_linear_speed {
                *val = -self.max_linear_speed;
                clamped = true;
            }
        }

        // Açısal hız bileşenlerini kırp ve NaN/Inf zehirlenmesine karşı koru
        for val in &mut cmd.angular {
            if !val.is_finite() {
                *val = 0.0;
                clamped = true;
            } else if *val > self.max_angular_speed {
                *val = self.max_angular_speed;
                clamped = true;
            } else if *val < -self.max_angular_speed {
                *val = -self.max_angular_speed;
                clamped = true;
            }
        }

        if clamped {
            warn!("Kinematik komut mekanik eşiklere kelepçelendi (Clamped)!");
        }

        clamped
    }

    /// Verilen [X, Y, Z] konumunun Geofence sınırları içinde olup olmadığını doğrular
    pub fn is_position_allowed(&self, x: f64, y: f64, z: f64) -> bool {
        // 1. Z (irtifa / derinlik) kontrolü
        if z < self.z_bounds.0 || z > self.z_bounds.1 {
            return false;
        }

        // 2. 2D Çokgen Geofence kontrolü
        if let Some(poly) = &self.boundary_polygon {
            let pt = Point::new(x, y);
            if !poly.contains(&pt) {
                return false;
            }
        }

        true
    }

    /// Verilen konumun bir eylem yasak alanında (Inhibit Zone) olup olmadığını denetler
    pub fn is_action_inhibited(&self, x: f64, y: f64) -> bool {
        let pt = Point::new(x, y);
        for zone in &self.inhibit_zones {
            if zone.contains(&pt) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo::{Coord, LineString};

    #[test]
    fn test_velocity_clamping() {
        let engine = ClampingEngine::new(None, (-10.0, 100.0), 10.0, 2.0);

        let mut cmd = KinematicCommand::new([15.0, -25.0, 5.0], [0.0, 5.0, -1.0], 1000);
        let was_clamped = engine.clamp_command(&mut cmd);

        assert!(was_clamped);
        assert_eq!(cmd.linear[0], 10.0);
        assert_eq!(cmd.linear[1], -10.0);
        assert_eq!(cmd.linear[2], 5.0);

        assert_eq!(cmd.angular[1], 2.0);
        assert_eq!(cmd.angular[2], -1.0);
    }

    #[test]
    fn test_geofence_polygon_and_inhibit() {
        // 0..100 x 0..100 karesi
        let square = Polygon::new(
            LineString::from(vec![
                Coord { x: 0.0, y: 0.0 },
                Coord { x: 100.0, y: 0.0 },
                Coord { x: 100.0, y: 100.0 },
                Coord { x: 0.0, y: 100.0 },
                Coord { x: 0.0, y: 0.0 },
            ]),
            vec![],
        );

        let mut engine = ClampingEngine::new(Some(square), (0.0, 50.0), 10.0, 2.0);

        // İçeride
        assert!(engine.is_position_allowed(50.0, 50.0, 25.0));

        // Dışarıda (X aşımı)
        assert!(!engine.is_position_allowed(105.0, 50.0, 25.0));

        // Dışarıda (İrtifa aşımı)
        assert!(!engine.is_position_allowed(50.0, 50.0, 55.0));

        // Inhibit Zone ekle (40..60 x 40..60)
        let inner_zone = Polygon::new(
            LineString::from(vec![
                Coord { x: 40.0, y: 40.0 },
                Coord { x: 60.0, y: 40.0 },
                Coord { x: 60.0, y: 60.0 },
                Coord { x: 40.0, y: 60.0 },
                Coord { x: 40.0, y: 40.0 },
            ]),
            vec![],
        );
        engine.add_inhibit_zone(inner_zone);

        assert!(engine.is_action_inhibited(50.0, 50.0));
        assert!(!engine.is_action_inhibited(20.0, 20.0));
    }
}
