use std::f64::consts::PI;

pub const WGS84_A: f64 = 6378137.0; // Ekvator yarıçapı (metre)
pub const WGS84_F: f64 = 1.0 / 298.257223563; // Basıklık
pub const WGS84_E2: f64 = 2.0 * WGS84_F - WGS84_F * WGS84_F; // Birinci eksantriklik karesi

/// WGS84 Küresel Coğrafi Koordinat Noktası
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeoPoint {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub alt_m: f64,
}

impl GeoPoint {
    pub fn new(lat_deg: f64, lon_deg: f64, alt_m: f64) -> Self {
        Self {
            lat_deg,
            lon_deg,
            alt_m,
        }
    }

    pub fn lat_rad(&self) -> f64 {
        self.lat_deg.to_radians()
    }

    pub fn lon_rad(&self) -> f64 {
        self.lon_deg.to_radians()
    }
}

/// Yerel Teğet Düzlem Koordinat Noktası (North-East-Down)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NedPoint {
    pub north_m: f64,
    pub east_m: f64,
    pub down_m: f64,
}

impl NedPoint {
    pub fn new(north_m: f64, east_m: f64, down_m: f64) -> Self {
        Self {
            north_m,
            east_m,
            down_m,
        }
    }

    pub fn horizontal_distance_to(&self, other: &NedPoint) -> f64 {
        let dn = self.north_m - other.north_m;
        let de = self.east_m - other.east_m;
        (dn * dn + de * de).sqrt()
    }

    pub fn distance_to(&self, other: &NedPoint) -> f64 {
        let dn = self.north_m - other.north_m;
        let de = self.east_m - other.east_m;
        let dd = self.down_m - other.down_m;
        (dn * dn + de * de + dd * dd).sqrt()
    }
}

/// Coğrafi koordinattan Yer Merkezli Yer Sabit (ECEF) koordinata dönüşüm
pub fn geodetic_to_ecef(geo: &GeoPoint) -> (f64, f64, f64) {
    let lat = geo.lat_rad();
    let lon = geo.lon_rad();
    let h = geo.alt_m;

    let sin_lat = lat.sin();
    let cos_lat = lat.cos();
    let sin_lon = lon.sin();
    let cos_lon = lon.cos();

    let n = WGS84_A / (1.0 - WGS84_E2 * sin_lat * sin_lat).sqrt();

    let x = (n + h) * cos_lat * cos_lon;
    let y = (n + h) * cos_lat * sin_lon;
    let z = (n * (1.0 - WGS84_E2) + h) * sin_lat;

    (x, y, z)
}

/// ECEF koordinatlarından WGS84 Geodetic koordinatına dönüşüm (Bowring algoritması)
pub fn ecef_to_geodetic(x: f64, y: f64, z: f64) -> GeoPoint {
    let b = WGS84_A * (1.0 - WGS84_F);
    let e2 = WGS84_E2;
    let e_prime2 = (WGS84_A * WGS84_A - b * b) / (b * b);

    let p = (x * x + y * y).sqrt();
    let theta = (z * WGS84_A).atan2(p * b);

    let lat = (z + e_prime2 * b * theta.sin().powi(3))
        .atan2(p - e2 * WGS84_A * theta.cos().powi(3));
    let lon = y.atan2(x);

    let sin_lat = lat.sin();
    let n = WGS84_A / (1.0 - e2 * sin_lat * sin_lat).sqrt();
    let alt = p / lat.cos() - n;

    GeoPoint::new(lat.to_degrees(), lon.to_degrees(), alt)
}

/// WGS84 Küresel Koordinatını referans orijine göre yerel NED teğet düzlemine dönüştürür.
pub fn geodetic_to_ned(point: &GeoPoint, origin: &GeoPoint) -> NedPoint {
    let (px, py, pz) = geodetic_to_ecef(point);
    let (ox, oy, oz) = geodetic_to_ecef(origin);

    let dx = px - ox;
    let dy = py - oy;
    let dz = pz - oz;

    let lat = origin.lat_rad();
    let lon = origin.lon_rad();

    let sin_lat = lat.sin();
    let cos_lat = lat.cos();
    let sin_lon = lon.sin();
    let cos_lon = lon.cos();

    let north = -sin_lat * cos_lon * dx - sin_lat * sin_lon * dy + cos_lat * dz;
    let east = -sin_lon * dx + cos_lon * dy;
    let down = -cos_lat * cos_lon * dx - cos_lat * sin_lon * dy - sin_lat * dz;

    NedPoint::new(north, east, down)
}

/// Yerel NED koordinatını referans orijine göre WGS84 küresel koordinatına dönüştürür.
pub fn ned_to_geodetic(ned: &NedPoint, origin: &GeoPoint) -> GeoPoint {
    let lat = origin.lat_rad();
    let lon = origin.lon_rad();

    let sin_lat = lat.sin();
    let cos_lat = lat.cos();
    let sin_lon = lon.sin();
    let cos_lon = lon.cos();

    let dx = -sin_lat * cos_lon * ned.north_m - sin_lon * ned.east_m - cos_lat * cos_lon * ned.down_m;
    let dy = -sin_lat * sin_lon * ned.north_m + cos_lon * ned.east_m - cos_lat * sin_lon * ned.down_m;
    let dz = cos_lat * ned.north_m - sin_lat * ned.down_m;

    let (ox, oy, oz) = geodetic_to_ecef(origin);
    ecef_to_geodetic(ox + dx, oy + dy, oz + dz)
}

/// Haversine formülü ile iki küresel koordinat arasındaki yüzey mesafesini hesaplar (metre).
pub fn haversine_distance(p1: &GeoPoint, p2: &GeoPoint) -> f64 {
    let lat1 = p1.lat_rad();
    let lon1 = p1.lon_rad();
    let lat2 = p2.lat_rad();
    let lon2 = p2.lon_rad();

    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    WGS84_A * c
}

/// P1 noktasından P2 noktasına yönelim kerteriz açısı (Bearing) (radyan, Kuzey=0, Saat yönü)
pub fn bearing(p1: &GeoPoint, p2: &GeoPoint) -> f64 {
    let lat1 = p1.lat_rad();
    let lon1 = p1.lon_rad();
    let lat2 = p2.lat_rad();
    let lon2 = p2.lon_rad();

    let dlon = lon2 - lon1;
    let y = dlon.sin() * lat2.cos();
    let x = lat1.cos() * lat2.sin() - lat1.sin() * lat2.cos() * dlon.cos();

    let brg = y.atan2(x);
    (brg + 2.0 * PI) % (2.0 * PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_geodetic_to_ecef_roundtrip() {
        let ankara = GeoPoint::new(39.9208, 32.8541, 938.0);
        let (x, y, z) = geodetic_to_ecef(&ankara);
        let recovered = ecef_to_geodetic(x, y, z);

        assert!((recovered.lat_deg - ankara.lat_deg).abs() < 1e-6);
        assert!((recovered.lon_deg - ankara.lon_deg).abs() < 1e-6);
        assert!((recovered.alt_m - ankara.alt_m).abs() < 1e-3);
    }

    #[test]
    fn test_geodetic_to_ned_and_inverse() {
        let origin = GeoPoint::new(41.0082, 28.9784, 50.0); // İstanbul
        let target = GeoPoint::new(41.0182, 28.9884, 100.0); // Yaklaşık 1.4 km mesafe

        let ned = geodetic_to_ned(&target, &origin);
        assert!(ned.north_m > 500.0);
        assert!(ned.east_m > 500.0);
        assert!((ned.down_m - -50.0).abs() < 0.2); // ~0.15m Dünya küreselliği payı

        let recovered = ned_to_geodetic(&ned, &origin);
        assert!((recovered.lat_deg - target.lat_deg).abs() < 1e-6);
        assert!((recovered.lon_deg - target.lon_deg).abs() < 1e-6);
        assert!((recovered.alt_m - target.alt_m).abs() < 1e-3);
    }

    #[test]
    fn test_haversine_and_bearing() {
        let p1 = GeoPoint::new(40.0, 30.0, 0.0);
        let p2 = GeoPoint::new(41.0, 30.0, 0.0); // Tam kuzeyde 1 derece

        let dist = haversine_distance(&p1, &p2);
        // 1 derece enlem yaklaşık 111 km
        assert!((dist - 111195.0).abs() < 500.0);

        let brg = bearing(&p1, &p2);
        assert!((brg - 0.0).abs() < 1e-4); // Tam kuzey = 0 radyan
    }
}
