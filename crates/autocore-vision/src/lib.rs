//! # autocore-vision
//!
//! Algılama, Çıkarım, Çoklu Hedef Takip ve Uzaysal İzdüşüm Çekirdeği.
//!
//! - `tracker`: ByteTrack / SORT çoklu nesne takipçisi ve Coasting (kör uçuş).
//! - `depth`: Pinhole kamera modeliyle 2D pikselden 3B metre koordinatına izdüşüm (DepthProjector).
//! - `iff`: Çoğunluk oylamalı dinamik IFF (Friend/Foe) ve parlama süzgeci.
//! - `mock`: SITL test ikizi ve sentetik kamera akışı (SyntheticCameraFeed).

pub mod depth;
pub mod iff;
pub mod lead_angle;
pub mod mock;
pub mod tracker;
pub mod types;

pub use depth::{CameraIntrinsics, DepthProjector};
pub use iff::DynamicIffFilter;
pub use lead_angle::{LeadAngleSolver, LeadSolution};
pub use mock::SyntheticCameraFeed;
pub use tracker::SortTracker;
pub use types::{
    BoundingBox, Detection, TargetClassification, Track, TrackState,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Doğrulama 1: Çoklu nesne takibi ID sürekliliği ve Coasting (Kör Uçuş) testi
    #[test]
    fn test_spatial_tracker_coasting() {
        let mut tracker = SortTracker::new(5, 2, 0.3); // max_age = 5, min_hits = 2
        let mut synth = SyntheticCameraFeed::new(1920.0, 1080.0);

        // 1. Kare: Hedef tespit edildi (Tentative)
        let frame1 = synth.generate_frame(false);
        let tracks1 = tracker.update(&frame1);
        assert_eq!(tracks1.len(), 1);
        let target_id = tracks1[0].id;

        // 2. Kare: Hedef tekrar görüldü (Confirmed)
        let frame2 = synth.generate_frame(false);
        let tracks2 = tracker.update(&frame2);
        assert_eq!(tracks2.len(), 1);
        assert_eq!(tracks2[0].id, target_id);
        assert_eq!(tracks2[0].state, TrackState::Confirmed);

        // 3..5 Kareler: Hedef direğin arkasına geçti (Occlusion - 3 kare yok)
        for _ in 0..3 {
            let empty_frame = synth.generate_frame(true);
            let tracks_coast = tracker.update(&empty_frame);
            assert_eq!(tracks_coast.len(), 1, "Hedef silinmemeli, coasting yapmalı");
            assert_eq!(tracks_coast[0].id, target_id);
            assert_eq!(tracks_coast[0].state, TrackState::Lost);
        }

        // 6. Kare: Hedef tekrar belirdi (Aynı ID ile devam etmeli!)
        let frame6 = synth.generate_frame(false);
        let tracks6 = tracker.update(&frame6);
        assert_eq!(tracks6.len(), 1);
        assert_eq!(tracks6[0].id, target_id, "Coasting sonrası ID korunmalı");
        assert_eq!(tracks6[0].state, TrackState::Confirmed);
    }

    /// Doğrulama 2: Kamera iç matrisi ile 2D pikselden 3B metre derinlik izdüşümü
    #[test]
    fn test_depth_projection_accuracy() {
        let intrinsics = CameraIntrinsics {
            fx: 1000.0,
            fy: 1000.0,
            cx: 500.0,
            cy: 500.0,
            image_width: 1000,
            image_height: 1000,
        };
        let projector = DepthProjector::new(intrinsics);

        // 1. Tam optik merkezde (u = cx = 500, v = cy = 500), Z = 10m derinlikte hedef:
        // Optik eksende X = 0.0, Y = 0.0, Z = 10.0 metre olmalı!
        let p3d_center = projector.pixel_to_3d(500.0, 500.0, 10.0);
        assert_eq!(p3d_center, [0.0, 0.0, 10.0]);

        // 2. Sağ üst köşede bir hedef: u = 700 (200 piksel sağ), v = 400 (100 piksel yukarı)
        // X = (700 - 500) * 10 / 1000 = 2.0 metre
        // Y = (400 - 500) * 10 / 1000 = -1.0 metre
        let p3d_offset = projector.pixel_to_3d(700.0, 400.0, 10.0);
        assert!((p3d_offset[0] - 2.0).abs() < 1e-4);
        assert!((p3d_offset[1] - (-1.0)).abs() < 1e-4);
        assert!((p3d_offset[2] - 10.0).abs() < 1e-4);

        // 3. Geriye piksel izdüşüm testi
        let pixel_reproj = projector.project_3d_to_pixel(p3d_offset).unwrap();
        assert!((pixel_reproj.0 - 700.0).abs() < 1e-3);
        assert!((pixel_reproj.1 - 400.0).abs() < 1e-3);
    }

    /// Doğrulama 3: Tek karelik parlamaların onaylı hedefe dönüşmemesi (Parlama Reddi)
    #[test]
    fn test_iff_optical_glare_rejection() {
        let mut iff = DynamicIffFilter::new(5, 3, 0.7); // 5 pencere, 3 asgari numune, %70 çoğunluk eşiği

        // Tek bir karede anlık sahte parlama geliyor
        let cls_single = iff.update(1, TargetClassification::Foe, 0.85);
        assert_eq!(
            cls_single,
            TargetClassification::Unknown,
            "Tek karelik parlama onaylanamaz"
        );

        // 2. ve 3. karelerde hedef görülmüyor / kayboluyor
        assert_eq!(iff.classify(1), TargetClassification::Unknown);
    }
}
