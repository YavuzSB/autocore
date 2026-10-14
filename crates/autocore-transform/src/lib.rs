//! # autocore-transform
//!
//! Koordinat Ağacı, SE(3) Lie Grubu, Geodezi ve Optik/Mekanik İzdüşüm Çekirdeği.
//!
//! - `TransformTree`: Çoklu sensör ve aktüatör çerçeveleri arası SE(3) dönüşüm ağı.
//! - `GeoConverter`: WGS84 <-> ECEF <-> NED küresel geodezi motoru (<1mm hassasiyet).
//! - `FrameProjector`: Bilgisayarlı görü (Z-ileri) ile mekanik gövde (X-ileri) eksen köprüsü.
//! - `se3`: Birim kuaterniyon ve 4x4 homojen dönüşüm rijit cisim matematiği.

pub mod error;
pub mod geo;
pub mod mock;
pub mod projector;
pub mod se3;
pub mod tree;
pub mod types;

pub use error::TransformError;
pub use geo::{GeoConverter, GeoPoint};
pub use mock::MockTransformBroadcaster;
pub use projector::FrameProjector;
pub use se3::{Quaternion, Transform3D};
pub use tree::TransformTree;
pub use types::{GnssRaw, LocalPose, TfStamped};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Doğrulama 1: WGS84 <-> ECEF <-> NED Dönüşümünde <1 mm Hata Toleransı
    #[test]
    fn test_geo_conversions() {
        // Referans Noktası: Ankara (39.920770° K, 32.854110° D, 938.0 m)
        let ref_pt = GeoPoint::new(39.920770, 32.854110, 938.0).expect("Geçerli ref nokta");

        // Test Noktası: Referanstan ~500m Kuzey, ~300m Doğu, ~-25m İrtifa (Yukarı)
        let target_pt =
            GeoPoint::new(39.925270, 32.857620, 963.0).expect("Geçerli hedef nokta");

        // 1. Geodetic -> ECEF -> Geodetic Çift Yönlü Doğruluk
        let ecef = GeoConverter::geodetic_to_ecef(&target_pt);
        let geo_recovered =
            GeoConverter::ecef_to_geodetic(ecef).expect("ECEF to Geodetic başarılı olmalı");

        let lat_err = (geo_recovered.lat_deg - target_pt.lat_deg).abs();
        let lon_err = (geo_recovered.lon_deg - target_pt.lon_deg).abs();
        let alt_err = (geo_recovered.alt_m - target_pt.alt_m).abs();

        // 1e-8 derece ~ 1.1 mm
        assert!(lat_err < 1e-8, "Enlem hatası 1e-8 dereceden küçük olmalı: {}", lat_err);
        assert!(lon_err < 1e-8, "Boylam hatası 1e-8 dereceden küçük olmalı: {}", lon_err);
        assert!(alt_err < 0.001, "İrtifa hatası 1 mm'den küçük olmalı: {} m", alt_err);

        // 2. Geodetic -> NED -> Geodetic Çift Yönlü Doğruluk
        let ned = GeoConverter::geodetic_to_ned(&target_pt, &ref_pt);
        let geo_from_ned =
            GeoConverter::ned_to_geodetic(ned, &ref_pt).expect("NED to Geodetic başarılı");

        let distance_err = GeoConverter::haversine_distance_m(&target_pt, &geo_from_ned);
        assert!(
            distance_err < 0.001,
            "Haversine geri dönüşüm mesafe hatası < 1mm olmalı: {} m",
            distance_err
        );

        // 3. NED -> ECEF -> NED Doğruluk
        let ecef_from_ned = GeoConverter::ned_to_ecef(ned, &ref_pt);
        let ned_recovered = GeoConverter::ecef_to_ned(ecef_from_ned, &ref_pt);

        for i in 0..3 {
            let diff = (ned[i] - ned_recovered[i]).abs();
            assert!(
                diff < 0.001,
                "NED eksen {} hatası < 1mm olmalı: {} m",
                i,
                diff
            );
        }
    }

    /// Doğrulama 2: Kamera Optik (Z-ileri, X-sağ, Y-aşağı) -> Mekanik Gövde (X-ileri, Y-sol, Z-yukarı)
    #[test]
    fn test_optical_to_mechanical_projection() {
        // Kameranın 10 metre tam önünde bir hedef: [0, 0, 10]
        let optical_pt = [0.0, 0.0, 10.0];
        let body_pt = FrameProjector::optical_to_body(optical_pt);

        // Mekanik gövdede X ileri olmalı: [10, 0, 0]
        assert_eq!(body_pt, [10.0, 0.0, 0.0]);

        // Geri dönüş testi
        let recovered_optical = FrameProjector::body_to_optical(body_pt);
        assert_eq!(recovered_optical, optical_pt);

        // Sağdaki hedef: [2.0, 0.0, 5.0] (2m sağ, 5m ileri)
        // Gövdede: X = 5.0 (ileri), Y = -2.0 (sağ = negatif Y), Z = 0.0
        let right_target = [2.0, 0.0, 5.0];
        let body_right = FrameProjector::optical_to_body(right_target);
        assert_eq!(body_right, [5.0, -2.0, 0.0]);

        // Transform3D eşdeğerlik testi
        let proj_tf = FrameProjector::optical_to_body_transform();
        let transformed_pt = proj_tf.transform_point(optical_pt);
        assert!(
            (transformed_pt[0] - 10.0).abs() < 1e-5
                && transformed_pt[1].abs() < 1e-5
                && transformed_pt[2].abs() < 1e-5,
            "Transform3D optical_to_body eşdeğer olmalı: {:?}",
            transformed_pt
        );
    }

    /// Doğrulama 3: SE(3) Lie Grubu Kuaterniyon ve Dönüşüm Bileşkesi
    #[test]
    fn test_se3_composition() {
        // T1: X yönünde +10 metre öteleme
        let t1 = Transform3D::from_xyz_rpy(10.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        // T2: Z ekseninde 90 derece Yaw dönme (pi/2)
        let t2 = Transform3D::from_xyz_rpy(
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            std::f32::consts::FRAC_PI_2,
        );

        // T_composed = T1 * T2
        let t_comp = t1.compose(&t2);

        // [1, 0, 0] noktasını dönüştür:
        // Önce T2 ile döner: [1, 0, 0] 90 derece dönünce [0, 1, 0] olur.
        // Sonra T1 ile ötelenir: [0, 1, 0] + [10, 0, 0] = [10, 1, 0].
        let pt = [1.0, 0.0, 0.0];
        let transformed = t_comp.transform_point(pt);

        assert!((transformed[0] - 10.0).abs() < 1e-5);
        assert!((transformed[1] - 1.0).abs() < 1e-5);
        assert!(transformed[2].abs() < 1e-5);

        // Ters dönüşüm testi: T * T^{-1} = Identity
        let t_inv = t_comp.inverse();
        let identity_chk = t_comp.compose(&t_inv);
        assert!(identity_chk.translation[0].abs() < 1e-5);
        assert!(identity_chk.translation[1].abs() < 1e-5);
        assert!(identity_chk.translation[2].abs() < 1e-5);
        assert!((identity_chk.rotation.w - 1.0).abs() < 1e-5);
    }

    /// Doğrulama 4: TransformTree Çerçeve Zincirleme ve Ortak Ata Çözümü
    #[test]
    fn test_transform_tree_chaining() {
        let tree = TransformTree::new();

        // Ağaç Yapısı:
        // map -> base_link (x: 5.0, y: 0.0, z: 0.0)
        // base_link -> camera_link (x: 0.5, y: 0.0, z: 0.2)
        // base_link -> turret_link (x: 0.0, y: 0.0, z: 0.4)
        tree.set_transform(
            "map",
            "base_link",
            Transform3D::from_xyz_rpy(5.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            1000,
        )
        .unwrap();

        tree.set_transform(
            "base_link",
            "camera_link",
            Transform3D::from_xyz_rpy(0.5, 0.0, 0.2, 0.0, 0.0, 0.0),
            1000,
        )
        .unwrap();

        tree.set_transform(
            "base_link",
            "turret_link",
            Transform3D::from_xyz_rpy(0.0, 0.0, 0.4, 0.0, 0.0, 0.0),
            1000,
        )
        .unwrap();

        // 1. camera_link orijinini map çerçevesine dönüştür
        // p_camera = [0, 0, 0] -> p_map = [5.5, 0.0, 0.2]
        let p_map = tree
            .transform_point("camera_link", "map", [0.0, 0.0, 0.0])
            .unwrap();
        assert!((p_map[0] - 5.5).abs() < 1e-5);
        assert!(p_map[1].abs() < 1e-5);
        assert!((p_map[2] - 0.2).abs() < 1e-5);

        // 2. camera_link'teki bir noktayı kardeş dal turret_link'e dönüştür (Ortak ata: base_link)
        // camera_link [0, 0, 0] -> base_link [0.5, 0.0, 0.2]
        // turret_link [0, 0, 0] base_link'e göre [0.0, 0.0, 0.4] olduğundan:
        // turret_link'e göre p = [0.5, 0.0, 0.2 - 0.4] = [0.5, 0.0, -0.2]
        let p_turret = tree
            .transform_point("camera_link", "turret_link", [0.0, 0.0, 0.0])
            .unwrap();
        assert!((p_turret[0] - 0.5).abs() < 1e-5);
        assert!(p_turret[1].abs() < 1e-5);
        assert!((p_turret[2] - (-0.2)).abs() < 1e-5);
    }

    /// Doğrulama 5: Döngüsel Bağımlılık Tespiti (Cycle Detection)
    #[test]
    fn test_cycle_detection() {
        let tree = TransformTree::new();
        tree.set_transform("A", "B", Transform3D::identity(), 100)
            .unwrap();
        tree.set_transform("B", "C", Transform3D::identity(), 100)
            .unwrap();

        // C -> A eklenirse döngü oluşmalı ve hata dönmeli
        let res = tree.set_transform("C", "A", Transform3D::identity(), 100);
        assert!(matches!(res, Err(TransformError::CyclicDependency(_))));

        // Öz döngü A -> A
        let self_res = tree.set_transform("A", "A", Transform3D::identity(), 100);
        assert!(matches!(self_res, Err(TransformError::CyclicDependency(_))));
    }

    /// Doğrulama 6: C-ABI Sıfır Kopyalama ve Bytemuck Hizalaması
    #[test]
    fn test_c_abi_and_bytemuck() {
        let pose = LocalPose::new([1.0, 2.0, 3.0], [1.0, 0.0, 0.0, 0.0], [0.5, 0.0, 0.0], 123456);
        let bytes = pose.as_bytes();
        assert_eq!(bytes.len(), 48);

        let parsed = LocalPose::try_from_bytes(bytes).expect("Pod parse başarılı olmalı");
        assert_eq!(*parsed, pose);

        let tf_stamped = TfStamped::new("map", "base_link", [1.0, 2.0, 3.0], [1.0, 0.0, 0.0, 0.0], 9999);
        assert_eq!(tf_stamped.parent_str(), "map");
        assert_eq!(tf_stamped.child_str(), "base_link");
        assert_eq!(tf_stamped.as_bytes().len(), 72);
    }

    /// Doğrulama 7: SITL Mock Broadcaster Simülasyon Koşusu
    #[test]
    fn test_mock_broadcaster_sitl() {
        let tree = Arc::new(TransformTree::new());
        let broadcaster = MockTransformBroadcaster::new(tree.clone());

        broadcaster.setup_default_robot_tree(100).unwrap();

        // 1.0 saniye adımı
        let pose = broadcaster.step_simulation(1.0, 200).unwrap();
        assert_eq!(pose.position[0], 2.0); // 2 m/s * 1.0s

        // map -> camera_link dönüşümünü doğrula
        let pt_cam = tree.transform_point("camera_link", "map", [0.0, 0.0, 0.0]).unwrap();
        assert!(pt_cam[0] > 2.0); // 2.0m + camera offset
    }
}
