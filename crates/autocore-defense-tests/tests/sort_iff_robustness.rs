//! Savunma Sanayii Algılama, Takip ve Angajman Sağlamlık Testleri
//!
//! Kapsam:
//! - Dinamik IFF (Dost/Düşman Tanıma) Optik Karıştırma ve Yanıltma Dayanımı
//! - Hipersonik Çapraz Uçuş Önleme (Lead-Angle Intercept Solver) Kinematik Çözüm Sınırları
//! - SORT Takipçisinin Sahte Hedef (False Alarm / Clutter) Eleme Kararlılığı

use autocore_defense_tests::DeterministicPrng;
use autocore_vision::iff::DynamicIffFilter;
use autocore_vision::lead_angle::LeadAngleSolver;
use autocore_vision::tracker::SortTracker;
use autocore_vision::types::{BoundingBox, Detection, TargetClassification, TrackState};

#[test]
fn test_iff_optical_jamming_and_glare_chattering_immunity() {
    let mut iff = DynamicIffFilter::new(10, 5, 0.7); // 10 kare geçmiş, min 5 örnek, 0.7 eşik
    let track_id = 42;

    // 1. Optik karıştırma / Işık parlaması simülasyonu: Ardışık zıt tespitler (Foe, Friend, Foe, Friend...)
    for step in 1..=20 {
        let raw_class = if step % 2 == 1 {
            TargetClassification::Foe
        } else {
            TargetClassification::Friend
        };
        let decision = iff.update(track_id, raw_class, 0.8);

        // Histerezis filtresi kararsızlıkta "Unknown" kalmalı; asla ani dost/düşman sıçraması yapmamalı
        assert_eq!(decision, TargetClassification::Unknown, "Optik titreşim altında IFF Unknown kalmalı!");
    }

    // 2. Güvenilir, kesintisiz düşman gözlemi (10 kare peş peşe Foe)
    let mut final_decision = TargetClassification::Unknown;
    for _ in 1..=10 {
        final_decision = iff.update(track_id, TargetClassification::Foe, 0.95);
    }
    assert_eq!(final_decision, TargetClassification::Foe, "Kararlı düşman gözleminde Foe onaylanmalı");

    // 3. Tek bir gürültülü kare (False positive Friend) düşman kilidini bozamaz
    let resilient_decision = iff.update(track_id, TargetClassification::Friend, 0.8);
    assert_eq!(resilient_decision, TargetClassification::Foe, "Tekil gürültülü kare Foe kilidini bozmamalı");
}

#[test]
fn test_hypersonic_crossing_lead_angle_solver() {
    // Hedef: 2000m uzakta, 500m irtifada, Mach 2 (680 m/s) hızla yana doğru çapraz uçan seyir füzesi
    let target_pos = [1000.0, 2000.0, 500.0];
    let target_vel = [680.0, 0.0, 0.0];

    // Önleyici füze / HSS mühimmatı hızı: Mach 3.5 (1190 m/s)
    let interceptor_pos = [0.0, 0.0, 0.0];
    let interceptor_speed = 1190.0;

    let solution = LeadAngleSolver::solve(target_pos, target_vel, interceptor_pos, interceptor_speed);
    assert!(solution.is_some(), "Mach 3.5 önleyici Mach 2 hedefi yakalayabilmeli");

    let sol = solution.unwrap();
    assert!(sol.time_of_flight_s > 0.0, "Uçuş süresi pozitif olmalı: {} s", sol.time_of_flight_s);
    assert!(sol.time_of_flight_s < 5.0, "Uçuş süresi makul sınırda olmalı (< 5s): {}", sol.time_of_flight_s);

    // Çarpışma noktasını doğrula: P_aim = P_target + V_target * t
    let expected_x = target_pos[0] + target_vel[0] * sol.time_of_flight_s;
    assert!((sol.aim_point[0] - expected_x).abs() < 1.0, "X ekseni önleme noktası tutarsız");

    // 2. Fiziksel olarak yakalanamaz hedef testi:
    // Önleyici hızı (200 m/s), hedefin hızından (680 m/s) çok düşükse çözüm imkansız olmalı
    let impossible = LeadAngleSolver::solve(target_pos, target_vel, interceptor_pos, 200.0);
    assert!(impossible.is_none(), "Daha yavaş önleyici çapraz hedef için None dönmeli");
}

#[test]
fn test_sort_tracker_cluttered_false_alarm_rejection() {
    let mut tracker = SortTracker::new(5, 3, 0.3); // max 5 yaş, min 3 isabet (onay), IoU 0.3
    let mut prng = DeterministicPrng::new(0xABCD_EF01_2345_6789);

    // 1. Tek karelik sahte tespitler (Clutter / Sahte hedefler)
    for _ in 0..30 {
        let x = (prng.next_f32() * 500.0) as f32;
        let y = (prng.next_f32() * 500.0) as f32;
        let false_alarm = Detection {
            bbox: BoundingBox::new(x, y, 20.0, 20.0),
            confidence: 0.85,
            class_id: 1,
            class_name: "vehicle".into(),
        };

        let tracks = tracker.update(&[false_alarm]);
        // Tekil karede rastgele beliren sahte hedefler ASLA onaylı iz (confirmed track) olamaz
        let confirmed: Vec<_> = tracks.iter().filter(|t| t.state == TrackState::Confirmed).collect();
        assert_eq!(confirmed.len(), 0, "Tekil sahte tespit onaylı ize terfi edemez!");
    }

    // 2. Kararlı gerçek hedef (En az 3 ardışık karede tutarlı hareket)
    let mut true_tracks = Vec::new();
    for i in 1..=5 {
        let det = Detection {
            bbox: BoundingBox::new(100.0 + i as f32 * 5.0, 100.0, 40.0, 40.0),
            confidence: 0.95,
            class_id: 2,
            class_name: "drone".into(),
        };
        true_tracks = tracker.update(&[det]);
    }

    // 3. kare ve sonrasında onaylanmış olmalı
    let confirmed: Vec<_> = true_tracks.into_iter().filter(|t| t.state == TrackState::Confirmed).collect();
    assert_eq!(confirmed.len(), 1, "Gerçek hedef 3 kare sonra onaylı ize dönüşmeli");
    assert_eq!(confirmed[0].hits, 5);
}
