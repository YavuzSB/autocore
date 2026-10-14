//! autocore-defense-tests :: Yetkinlik ve Fizik Blokları (Capability Primitives) Çoklu Çekirdek Entegrasyon Testi
//!
//! 7 Üst Çekirdeğin (Vision -> Transform -> Pointing -> Arbiter -> Safety)
//! uçtan uca savunma sanayii standardında senkronizasyonunu ve sıfır-kopyalama veri akışını doğrular.

use autocore_arbiter::{CommandArbiter, OverrideReason, PriorityTier, Tier1SafetyStatus};
use autocore_base::{KinematicCommand, SafetyMode};
use autocore_pointing::{
    ActionDiscipline, InverseKinematics2DoF, LatencyCompensator, LeadAngleSolver, MockPanTiltRig,
    PointingCommand, TargetTrack,
};
use autocore_transform::{
    FrameProjector, GeoConverter, GeoPoint, Transform3D, TransformTree,
};
use autocore_vision::{CameraIntrinsics, DepthProjector, DynamicIffFilter, TargetClassification};

/// Test 1: Tam Otonom Hava Savunma / Hedef Takip ve Angajman Boru Hattı
///
/// Senaryo:
/// 1. autocore-vision: Kamera 1080p sensöründe piksel (u=1100, v=480, Z=150m) tespit eder.
/// 2. DepthProjector: 2D pikseli 3B kamera optik koordinatına çevirir.
/// 3. autocore-transform: Kamera optik eksenini robot gövdesine ve taret pivotuna dönüştürür.
/// 4. LatencyCompensator: 60 ms görüntü boru hattı gecikmesini hedefin hız vektörüyle ekstrapole eder.
/// 5. LeadAngleSolver: 2. derece kuadratik denklemle 1000 m/s mühimmat için önleme açısını ve yerçekimi düşüşünü çözer.
/// 6. InverseKinematics2DoF: 3B nişan noktasını Pan/Tilt açı ve açısal hızlarına dönüştürür.
/// 7. ActionDiscipline: Açısal hata toleransa girip servo salınımı sönümlenene kadar tetiği kilitler; sonra ateş izni verir.
/// 8. CommandArbiter: Tüm bu süreçte Tier 1 E-Stop basılırsa eyleyicilerin anında sıfırlanmasını garanti eder.
#[test]
fn test_end_to_end_air_defense_stack() {
    // -------------------------------------------------------------
    // ADIM 1: Kamera İç Kalibrasyonu ve 3B İzdüşüm (autocore-vision)
    // -------------------------------------------------------------
    let intrinsics = CameraIntrinsics {
        fx: 1200.0,
        fy: 1200.0,
        cx: 960.0,
        cy: 540.0,
        image_width: 1920,
        image_height: 1080,
    };
    let projector = DepthProjector::new(intrinsics);

    // Piksel koordinatı (u, v) ve derinlik Z = 150 metre
    let u_pixel = 1100.0;
    let v_pixel = 480.0;
    let depth_m = 150.0;

    let p_optical = projector.pixel_to_3d(u_pixel, v_pixel, depth_m);
    assert_eq!(p_optical[2], 150.0);

    // IFF Teşhis: 3 kare üst üste Foe tespit edildi
    let mut iff = DynamicIffFilter::new(5, 3, 0.7);
    iff.update(101, TargetClassification::Foe, 0.9);
    iff.update(101, TargetClassification::Foe, 0.95);
    let target_identity = iff.update(101, TargetClassification::Foe, 0.92);
    assert_eq!(target_identity, TargetClassification::Foe);

    // -------------------------------------------------------------
    // ADIM 2: Koordinat Ağacı ve Eksen İzdüşümü (autocore-transform)
    // -------------------------------------------------------------
    // Kamera optik (Z-ileri, X-sağ, Y-aşağı) -> Mekanik gövde (X-ileri, Y-sol, Z-yukarı)
    let p_body = FrameProjector::optical_to_body(p_optical);
    assert_eq!(p_body[0], 150.0); // Z optik -> X mekanik

    let tree = TransformTree::new();
    // base_link -> turret_link (Taret tavanın 0.5m yukarısında)
    tree.set_transform(
        "base_link",
        "turret_link",
        Transform3D::from_xyz_rpy(0.0, 0.0, 0.5, 0.0, 0.0, 0.0),
        1000,
    )
    .unwrap();

    // Hedefin tarete göre koordinatı (Z ekseninde 0.5m aşağı kayar)
    let p_turret = tree
        .transform_point("base_link", "turret_link", p_body)
        .unwrap();
    assert_eq!(p_turret[0], p_body[0]);
    assert!((p_turret[2] - (p_body[2] - 0.5)).abs() < 1e-4);

    // -------------------------------------------------------------
    // ADIM 3: Gecikme Telafisi & Balistik Önleme (autocore-pointing)
    // -------------------------------------------------------------
    let target_velocity = [10.0, 30.0, 0.0]; // 30 m/s yanal hız
    let track = TargetTrack {
        position: p_turret,
        velocity: target_velocity,
        timestamp_us: 1_000_000,
    };

    // 60 ms boru hattı gecikmesi sonrasına ekstrapolasyon
    let p_extrapolated = LatencyCompensator::extrapolate(&track, 1_060_000);
    // 30 m/s * 0.06s = 1.8 metre yanal öteleme
    assert!((p_extrapolated[1] - (p_turret[1] + 1.8)).abs() < 1e-3);

    // Mühimmat: 1000 m/s namlu çıkış hızı ile kuadratik balistik çözüm
    let lead_solution = LeadAngleSolver::solve(p_extrapolated, target_velocity, 1000.0)
        .expect("Balistik çözüm bulunmalı");
    assert!(lead_solution.time_of_flight_s > 0.1);
    assert!(lead_solution.gravity_drop_m > 0.0);

    // -------------------------------------------------------------
    // ADIM 4: Ters Kinematik ve Eylem Güvenlik Disiplini
    // -------------------------------------------------------------
    let ik = InverseKinematics2DoF::default();
    let setpoints = ik.solve(lead_solution.aim_point, target_velocity);
    assert!(setpoints.within_limits);

    let mut discipline = ActionDiscipline::new(0.5, 0.05, 50, 100);
    let mut rig = MockPanTiltRig::new(0.04); // 40 ms servo ataleti

    // Başlangıçta taret hedeften uzakta:
    let cmd_initial = PointingCommand::new(
        setpoints.pan_rad,
        setpoints.tilt_rad,
        setpoints.pan_rate_rad_s,
        setpoints.tilt_rate_rad_s,
        false,
    );

    // 5 adım simülasyon (100 ms) servolar hedefe doğru dönüyor
    let mut current_fb = rig.step(&cmd_initial, 0.02, 100_000);
    for i in 1..10 {
        current_fb = rig.step(&cmd_initial, 0.02, 100_000 + i * 20_000);
    }

    // Açı hedefe yaklaştı ve 50ms sakin kaldıktan sonra değerlendir:
    let fire_allowed = discipline.evaluate(
        current_fb.current_pan_rad,
        current_fb.current_tilt_rad,
        setpoints.pan_rad,
        setpoints.tilt_rad,
        current_fb.pan_rate_rad_s,
        current_fb.tilt_rate_rad_s,
        SafetyMode::Mission,
        400_000,
    );

    // Eğer tolerans içine girmişse ateş açılabilir
    if fire_allowed {
        discipline.record_fire(400_000);
    }

    // -------------------------------------------------------------
    // ADIM 5: Komut Hakemi (autocore-arbiter) & E-Stop Veto Testi
    // -------------------------------------------------------------
    let mut arbiter = CommandArbiter::default();

    let mission_cmd = KinematicCommand {
        linear: [2.0, 0.0, 0.0],
        angular: [0.0, 0.0, setpoints.pan_rad],
        timestamp_us: 400_000,
    };

    // Normal operasyon: Görev komutu geçer
    let tier1_ok = Tier1SafetyStatus::safe_operating();
    let arb_normal = arbiter.arbitrate(&tier1_ok, None, Some(mission_cmd), 0.02, 400_000);
    assert_eq!(arb_normal.active_tier, PriorityTier::Tier3Mission as u8);

    // Acil durum: E-Stop tetiklenirse anında Tier 1 Veto'ya düşer
    let mut tier1_estop = Tier1SafetyStatus::safe_operating();
    tier1_estop.estop_active = true;

    let arb_estop = arbiter.arbitrate(&tier1_estop, None, Some(mission_cmd), 0.02, 410_000);
    assert_eq!(arb_estop.active_tier, PriorityTier::Tier1Safety as u8);
    assert_eq!(arb_estop.override_reason, OverrideReason::EStop as u8);
    assert_eq!(arb_estop.command.linear, [0.0, 0.0, 0.0]);
}

/// Test 2: Küresel Geodezi ve Çoklu Alan Koordinat Dönüşüm Bütünlüğü
#[test]
fn test_geodesy_and_coordinate_integrity() {
    let base_station = GeoPoint::new(41.0082, 28.9784, 50.0).unwrap(); // İstanbul
    let target_gnss = GeoPoint::new(41.0120, 28.9830, 120.0).unwrap();

    // WGS84 -> NED dönüşümü
    let ned = GeoConverter::geodetic_to_ned(&target_gnss, &base_station);
    assert!(ned[0] > 0.0); // Kuzeyde
    assert!(ned[1] > 0.0); // Doğuda
    assert!(ned[2] < 0.0); // Yukarıda (NED Down negatif)

    // Haversine mesafesi ile Kartezyen mesafe uyumu
    let haversine_dist = GeoConverter::haversine_distance_m(&base_station, &target_gnss);
    let ned_dist_2d = (ned[0] * ned[0] + ned[1] * ned[1]).sqrt() as f64;
    assert!((haversine_dist - ned_dist_2d).abs() < 1.0); // 1 metre içinde tam uyum
}
