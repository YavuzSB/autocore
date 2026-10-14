//! autocore-transform :: Geodezi ve Küresel Koordinat Dönüşüm Motoru
//!
//! WGS84 Elipsoiti, ECEF ve Yerel Teğet Düzlem (NED/ENU) koordinat dönüşümleri.
//! Savunma ve seyrüsefer standardı: <1 mm yuvarlama hassasiyeti.

use crate::error::TransformError;
use serde::{Deserialize, Serialize};

/// WGS84 Elipsoit Sabitleri
pub const WGS84_A: f64 = 6378137.0; // Yarı büyük eksen (metre)
pub const WGS84_F: f64 = 1.0 / 298.257223563; // Basıklık (flattening)
pub const WGS84_B: f64 = WGS84_A * (1.0 - WGS84_F); // Yarı küçük eksen (6356752.314245 m)
pub const WGS84_E2: f64 = 2.0 * WGS84_F - WGS84_F * WGS84_F; // Birinci eksantriklik karesi (~0.00669438)
pub const WGS84_E_PRIME2: f64 = (WGS84_A * WGS84_A - WGS84_B * WGS84_B) / (WGS84_B * WGS84_B); // İkinci eksantriklik karesi (~0.00673950)

/// Küresel Coğrafi Konum Noktası (WGS84)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoPoint {
    /// Enlem (Latitude) - derece cinsinden [-90.0, 90.0]
    pub lat_deg: f64,
    /// Boylam (Longitude) - derece cinsinden [-180.0, 180.0]
    pub lon_deg: f64,
    /// Elipsoit Üstü İrtifa (Altitude) - metre
    pub alt_m: f64,
}

impl Default for GeoPoint {
    fn default() -> Self {
        Self {
            lat_deg: 0.0,
            lon_deg: 0.0,
            alt_m: 0.0,
        }
    }
}

impl GeoPoint {
    pub fn new(lat_deg: f64, lon_deg: f64, alt_m: f64) -> Result<Self, TransformError> {
        if !lat_deg.is_finite() || !lon_deg.is_finite() || !alt_m.is_finite() {
            return Err(TransformError::NumericalError(
                "Geçersiz NaN/Sonsuz coğrafi koordinat".to_string(),
            ));
        }
        if lat_deg < -90.0 || lat_deg > 90.0 {
            return Err(TransformError::NumericalError(format!(
                "Enlem sınır dışı: {} ([-90, 90] bekleniyor)",
                lat_deg
            )));
        }
        if lon_deg < -180.0 || lon_deg > 180.0 {
            return Err(TransformError::NumericalError(format!(
                "Boylam sınır dışı: {} ([-180, 180] bekleniyor)",
                lon_deg
            )));
        }
        Ok(Self {
            lat_deg,
            lon_deg,
            alt_m,
        })
    }
}

/// Geodezi ve Koordinat Dönüştürücü Motoru
pub struct GeoConverter;

impl GeoConverter {
    /// WGS84 Coğrafi koordinatları ECEF (Earth-Centered, Earth-Fixed) [X, Y, Z] metreye çevirir
    pub fn geodetic_to_ecef(geo: &GeoPoint) -> [f64; 3] {
        let lat_rad = geo.lat_deg.to_radians();
        let lon_rad = geo.lon_deg.to_radians();
        let sin_lat = lat_rad.sin();
        let cos_lat = lat_rad.cos();
        let sin_lon = lon_rad.sin();
        let cos_lon = lon_rad.cos();

        // Normal eğrilik yarıçapı N(phi)
        let n = WGS84_A / (1.0 - WGS84_E2 * sin_lat * sin_lat).sqrt();

        let x = (n + geo.alt_m) * cos_lat * cos_lon;
        let y = (n + geo.alt_m) * cos_lat * sin_lon;
        let z = (n * (1.0 - WGS84_E2) + geo.alt_m) * sin_lat;

        [x, y, z]
    }

    /// ECEF [X, Y, Z] metre koordinatlarını WGS84 Coğrafi koordinata çevirir (Bowring Yöntemi)
    pub fn ecef_to_geodetic(ecef: [f64; 3]) -> Result<GeoPoint, TransformError> {
        let x = ecef[0];
        let y = ecef[1];
        let z = ecef[2];

        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return Err(TransformError::NumericalError(
                "Geçersiz NaN/Sonsuz ECEF koordinatı".to_string(),
            ));
        }

        let p = (x * x + y * y).sqrt();

        // Kutup noktaları özel durumu
        if p < 1e-10 {
            let lat_deg = if z >= 0.0 { 90.0 } else { -90.0 };
            let alt_m = z.abs() - WGS84_B;
            return Ok(GeoPoint {
                lat_deg,
                lon_deg: 0.0,
                alt_m,
            });
        }

        let theta = (z * WGS84_A).atan2(p * WGS84_B);
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        let sin3_theta = sin_theta * sin_theta * sin_theta;
        let cos3_theta = cos_theta * cos_theta * cos_theta;

        let lat_rad = (z + WGS84_E_PRIME2 * WGS84_B * sin3_theta)
            .atan2(p - WGS84_E2 * WGS84_A * cos3_theta);
        let lon_rad = y.atan2(x);

        let sin_lat = lat_rad.sin();
        let cos_lat = lat_rad.cos();
        let n = WGS84_A / (1.0 - WGS84_E2 * sin_lat * sin_lat).sqrt();

        // Sayısal kararlılık için kutba yakın bölgelerde z üzerinden irtifa
        let alt_m = if lat_rad.abs() < 80.0_f64.to_radians() {
            p / cos_lat - n
        } else {
            z / sin_lat - n * (1.0 - WGS84_E2)
        };

        Ok(GeoPoint {
            lat_deg: lat_rad.to_degrees(),
            lon_deg: lon_rad.to_degrees(),
            alt_m,
        })
    }

    /// ECEF koordinatını referans coğrafi konuma göre yerel NED (North, East, Down) metreye çevirir
    pub fn ecef_to_ned(ecef: [f64; 3], ref_geo: &GeoPoint) -> [f32; 3] {
        let ref_ecef = Self::geodetic_to_ecef(ref_geo);
        let dx = ecef[0] - ref_ecef[0];
        let dy = ecef[1] - ref_ecef[1];
        let dz = ecef[2] - ref_ecef[2];

        let lat_rad = ref_geo.lat_deg.to_radians();
        let lon_rad = ref_geo.lon_deg.to_radians();
        let s_lat = lat_rad.sin();
        let c_lat = lat_rad.cos();
        let s_lon = lon_rad.sin();
        let c_lon = lon_rad.cos();

        let north = -s_lat * c_lon * dx - s_lat * s_lon * dy + c_lat * dz;
        let east = -s_lon * dx + c_lon * dy;
        let down = -c_lat * c_lon * dx - c_lat * s_lon * dy - s_lat * dz;

        [north as f32, east as f32, down as f32]
    }

    /// Yerel NED metre koordinatını referans konuma göre ECEF koordinatına çevirir
    pub fn ned_to_ecef(ned: [f32; 3], ref_geo: &GeoPoint) -> [f64; 3] {
        let ref_ecef = Self::geodetic_to_ecef(ref_geo);

        let lat_rad = ref_geo.lat_deg.to_radians();
        let lon_rad = ref_geo.lon_deg.to_radians();
        let s_lat = lat_rad.sin();
        let c_lat = lat_rad.cos();
        let s_lon = lon_rad.sin();
        let c_lon = lon_rad.cos();

        let n = ned[0] as f64;
        let e = ned[1] as f64;
        let d = ned[2] as f64;

        let dx = -s_lat * c_lon * n - s_lon * e - c_lat * c_lon * d;
        let dy = -s_lat * s_lon * n + c_lon * e - c_lat * s_lon * d;
        let dz = c_lat * n - s_lat * d;

        [ref_ecef[0] + dx, ref_ecef[1] + dy, ref_ecef[2] + dz]
    }

    /// WGS84 Coğrafi koordinatını doğrudan referans konuma göre yerel NED metreye çevirir
    pub fn geodetic_to_ned(geo: &GeoPoint, ref_geo: &GeoPoint) -> [f32; 3] {
        let ecef = Self::geodetic_to_ecef(geo);
        Self::ecef_to_ned(ecef, ref_geo)
    }

    /// Yerel NED metre koordinatını doğrudan WGS84 Coğrafi koordinatına çevirir
    pub fn ned_to_geodetic(ned: [f32; 3], ref_geo: &GeoPoint) -> Result<GeoPoint, TransformError> {
        let ecef = Self::ned_to_ecef(ned, ref_geo);
        Self::ecef_to_geodetic(ecef)
    }

    /// İki coğrafi nokta arasındaki büyük daire (Haversine) mesafesi (metre)
    pub fn haversine_distance_m(p1: &GeoPoint, p2: &GeoPoint) -> f64 {
        const EARTH_RADIUS_M: f64 = 6371000.0;
        let d_lat = (p2.lat_deg - p1.lat_deg).to_radians();
        let d_lon = (p2.lon_deg - p1.lon_deg).to_radians();
        let lat1 = p1.lat_deg.to_radians();
        let lat2 = p2.lat_deg.to_radians();

        let sin_dlat_2 = (d_lat * 0.5).sin();
        let sin_dlon_2 = (d_lon * 0.5).sin();

        let a = sin_dlat_2 * sin_dlat_2 + lat1.cos() * lat2.cos() * sin_dlon_2 * sin_dlon_2;
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        EARTH_RADIUS_M * c
    }

    /// İki nokta arasındaki kerteriz / yön açısı (Bearing) [0, 360) derece
    pub fn bearing_deg(from: &GeoPoint, to: &GeoPoint) -> f64 {
        let lat1 = from.lat_deg.to_radians();
        let lat2 = to.lat_deg.to_radians();
        let d_lon = (to.lon_deg - from.lon_deg).to_radians();

        let y = d_lon.sin() * lat2.cos();
        let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * d_lon.cos();

        let bearing_rad = y.atan2(x);
        let bearing_deg = bearing_rad.to_degrees();
        (bearing_deg + 360.0) % 360.0
    }
}
