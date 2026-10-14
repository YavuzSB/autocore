//! autocore-aerial :: 3B Hava Sahası Kafesi (AirGeofence)
//!
//! İrtifa tavanı/tabanı ve 2D poligon sınırlarını denetler;
//! ihlal anında otomatik RTL (Return-to-Launch) tetikler.

use crate::types::AutopilotSetpoint;

/// 3B Hava Sahası Güvenlik Kafesi
#[derive(Debug, Clone)]
pub struct AirGeofence {
    /// Minimum irtifa (metre)
    pub min_altitude_m: f32,
    /// Maksimum yasal irtifa tavanı (metre) - tipik 120.0m (SHGM / FAA limiti)
    pub max_altitude_m: f32,
    /// 2D Poligon sınır köşe noktaları [[North, East], ...]
    pub polygon_vertices_ned: Vec<[f32; 2]>,
}

impl Default for AirGeofence {
    fn default() -> Self {
        Self {
            min_altitude_m: 2.0,
            max_altitude_m: 120.0,
            polygon_vertices_ned: Vec::new(),
        }
    }
}

/// Geofence Denetim Sonucu
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GeofenceStatus {
    Safe,
    AltitudeFloorBreach(f32),
    AltitudeCeilingBreach(f32),
    PolygonBoundaryBreach,
}

impl AirGeofence {
    pub fn new(min_alt: f32, max_alt: f32, polygon: Vec<[f32; 2]>) -> Self {
        Self {
            min_altitude_m: min_alt,
            max_altitude_m: max_alt,
            polygon_vertices_ned: polygon,
        }
    }

    /// 3B konumu [North, East, Altitude_m] denetler
    pub fn check_position(&self, pos_ned: [f32; 2], altitude_m: f32) -> GeofenceStatus {
        if altitude_m > self.max_altitude_m {
            return GeofenceStatus::AltitudeCeilingBreach(altitude_m);
        }
        if altitude_m < self.min_altitude_m {
            return GeofenceStatus::AltitudeFloorBreach(altitude_m);
        }

        if !self.polygon_vertices_ned.is_empty() && !self.is_inside_polygon(pos_ned) {
            return GeofenceStatus::PolygonBoundaryBreach;
        }

        GeofenceStatus::Safe
    }

    /// Verilen komutu denetler: İhlal varsa RTL moduna çevirir
    pub fn enforce(
        &self,
        command: AutopilotSetpoint,
        pos_ned: [f32; 2],
        altitude_m: f32,
    ) -> (AutopilotSetpoint, GeofenceStatus) {
        let status = self.check_position(pos_ned, altitude_m);
        match status {
            GeofenceStatus::Safe => (command, status),
            _ => (AutopilotSetpoint::rtl(), status),
        }
    }

    /// Ray-Casting Algoritması ile 2D nokta-poligon içi testi
    fn is_inside_polygon(&self, pt: [f32; 2]) -> bool {
        let n = self.polygon_vertices_ned.len();
        if n < 3 {
            return true; // Poligon tanımlı değilse sınır yok sayılır
        }

        let mut inside = false;
        let x = pt[1]; // East
        let y = pt[0]; // North

        let mut j = n - 1;
        for i in 0..n {
            let xi = self.polygon_vertices_ned[i][1];
            let yi = self.polygon_vertices_ned[i][0];
            let xj = self.polygon_vertices_ned[j][1];
            let yj = self.polygon_vertices_ned[j][0];

            let intersect = ((yi > y) != (yj > y)) && (x < (xj - xi) * (y - yi) / (yj - yi) + xi);
            if intersect {
                inside = !inside;
            }
            j = i;
        }

        inside
    }
}
