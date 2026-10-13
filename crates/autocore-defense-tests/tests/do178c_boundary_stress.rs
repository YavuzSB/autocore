//! DO-178C (Uçuş Yazılımları Doğrulama Standardı) Sınır Değer ve Tekillik Testleri
//!
//! Kapsam:
//! - WGS84 Coğrafi Kutup Noktaları (+90° / -90° Enlem) ve Antimeridyen (±180° Boylam) Sınırları
//! - Aşırı İrtifa (-413m Ölü Deniz'den +25.000m Stratosfere)
//! - Gimbal Lock (Yalpalama Tekilliği ±90° Pitch) ve Yüksek Açısal Hız (500 rad/s) Kuaterniyon Değişmezliği
//! - Konkav Çokgen Geofence Köşe ve Sınır Kırpma Dayanımı
//! - 2D Maliyet Haritası Izgara Dışı (Out of Bounds) Sınır Aşımı Güvenliği

use autocore_ground::costmap::LocalCostmap2D;
use autocore_guidance::coordinates::{ecef_to_geodetic, geodetic_to_ecef, GeoPoint};
use autocore_guidance::state::{Quat, StateEstimator6DoF};
use autocore_safety::clamping::ClampingEngine;
use geo::{LineString, Polygon};

#[test]
fn test_do178c_wgs84_poles_antimeridian_and_stratosphere() {
    let extreme_coords: Vec<(&str, f64, f64, f64)> = vec![
        ("Kuzey Kutbu", 90.0, 0.0, 0.0),
        ("Güney Kutbu", -90.0, 0.0, 0.0),
        ("Antimeridyen Doğu", 0.0, 180.0, 100.0),
        ("Antimeridyen Batı", 0.0, -180.0, 100.0),
        ("Ölü Deniz (Deniz Altı İrtifa)", 31.5, 35.5, -413.0),
        ("Stratosferik İHA İrtifası", 39.9, 32.8, 25_000.0),
    ];

    for (name, lat, lon, alt) in extreme_coords {
        let pt = GeoPoint::new(lat, lon, alt);
        let (x, y, z) = geodetic_to_ecef(&pt);
        assert!(x.is_finite() && y.is_finite() && z.is_finite(), "{}: ECEF koordinatları sonlu olmalı", name);

        let r_pt = ecef_to_geodetic(x, y, z);
        assert!(r_pt.lat_deg.is_finite() && r_pt.lon_deg.is_finite() && r_pt.alt_m.is_finite(), "{}: Geri dönüşüm sonlu olmalı", name);

        // Enlem ve irtifa hata toleransları (milimetre ve mikroderece hassasiyeti)
        let lat_err = (r_pt.lat_deg - lat).abs();
        let alt_err = (r_pt.alt_m - alt).abs();

        assert!(lat_err < 1e-5, "{}: Enlem hatası çok yüksek: {}", name, lat_err);
        assert!(alt_err < 0.01, "{}: İrtifa hatası 1 cm'den az olmalı: {} m", name, alt_err);

        // Kutuplarda boylam tanımsızdır, diğer yerlerde kontrol et
        if lat.abs() < 89.9 {
            let mut lon_err = (r_pt.lon_deg - lon).abs();
            if lon_err > 359.0 {
                lon_err = (lon_err - 360.0).abs();
            }
            assert!(lon_err < 1e-4, "{}: Boylam hatası çok yüksek: {}", name, lon_err);
        }
    }
}

#[test]
fn test_do178c_quaternion_gimbal_lock_and_high_tumbling() {
    // 1. Gimbal Lock Singularity: Pitch = ±90.0° (Euler kuaterniyona çevrilirken tekillik olmamalı)
    let gimbal_cases = vec![
        (0.0f32, std::f32::consts::FRAC_PI_2, 0.0f32),      // +90 derece pitch
        (0.0f32, -std::f32::consts::FRAC_PI_2, 0.0f32),     // -90 derece pitch
        (0.5f32, std::f32::consts::FRAC_PI_2, 1.2f32),      // Roll ve yaw eşlikli
    ];

    for (roll, pitch, yaw) in gimbal_cases {
        let q = Quat::from_euler(roll, pitch, yaw);
        let norm = (q.w * q.w + q.x * q.x + q.y * q.y + q.z * q.z).sqrt();
        assert!((norm - 1.0).abs() < 1e-5, "Kuaterniyon birim normu bozulmamalı: {}", norm);

        // Euler'e geri dönüşümde pitch değeri korunmalı
        let (_, r_pitch, _) = q.to_euler();
        assert!((r_pitch - pitch).abs() < 1e-4, "Pitch tekilliği hatası: {} vs {}", r_pitch, pitch);
    }

    // 2. Yüksek Dönüş Hızı (500 rad/s / ~28.600 deg/s takla/spin) altında kuaterniyon integrasyonu
    let mut estimator = StateEstimator6DoF::new();
    let dt = 0.001; // 1 kHz ataletsel adım

    for step in 1..=1000 {
        let gyro = [500.0, -300.0, 200.0];
        let accel = [0.0, 0.0, -9.81];
        estimator.integrate(gyro, accel, dt, step * 1000);

        let q = estimator.current_state().quaternion;
        let norm = (q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]).sqrt();
        assert!((norm - 1.0).abs() < 1e-4, "Aşırı dönüşte kuaterniyon normalize kalmalı: norm={}", norm);
    }
}

#[test]
fn test_do178c_concave_geofence_corner_and_boundary_vertex_clamping() {
    // İçbükey (Konkav / L-şekilli) karmaşık güvenlik poligonu
    let poly = Polygon::new(
        LineString::from(vec![
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 5.0),
            (5.0, 5.0),
            (5.0, 10.0),
            (0.0, 10.0),
            (0.0, 0.0),
        ]),
        vec![],
    );

    let clamping = ClampingEngine::new(Some(poly), (0.0, 50.0), 10.0, 2.0);

    // İçerideki noktalar geçerli
    assert!(clamping.is_position_allowed(2.0, 2.0, 10.0));
    assert!(clamping.is_position_allowed(2.0, 8.0, 10.0));

    // Konkav çentik alanı (X=8, Y=8) poligonun DIŞINDADIR
    assert!(!clamping.is_position_allowed(8.0, 8.0, 10.0), "Konkav çentik dışarıda olmalı");

    // Aşırı uzak koordinat sıçraması (X=50,000m)
    assert!(!clamping.is_position_allowed(50_000.0, 50_000.0, 10.0));

    // Negatif veya sınır dışı irtifa
    assert!(!clamping.is_position_allowed(2.0, 2.0, -5.0), "Negatif irtifa yasak olmalı");
    assert!(!clamping.is_position_allowed(2.0, 2.0, 55.0), "Tavan üstü irtifa yasak olmalı");
}

#[test]
fn test_do178c_costmap_boundary_out_of_bounds_resilience() {
    let mut map = LocalCostmap2D::new(50, 50, 0.1); // 5m x 5m harita (-2.5 .. +2.5m)

    // Harita dışındaki aşırı koordinatlar (1000m uzakta engel tespiti, NaN/Inf)
    let oob_coords = vec![
        (1000.0, 1000.0),
        (-500.0, 0.0),
        (0.0, -800.0),
        (f64::MAX, f64::MIN),
        (f64::NAN, 0.0),
        (0.0, f64::INFINITY),
    ];

    for (x, y) in oob_coords {
        let grid_pos = map.metric_to_grid(x, y);
        assert!(grid_pos.is_none(), "Harita dışı metrik koordinat None dönmeli: ({}, {})", x, y);

        // Engel olarak işaretlenmeye çalışıldığında çökmemeli (Panic olmamalı)
        map.set_obstacle(x, y);
    }
}
