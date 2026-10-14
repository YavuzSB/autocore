//! # autocore-pointing
//!
//! Yönlendirme, Taret, Gimbal, Balistik Önleme ve Angajman Disiplini Çekirdeği.
//!
//! - `LatencyCompensator`: Görüntü/boru hattı gecikmesini hız vektörüyle telafi eder.
//! - `LeadAngleSolver`: 2. derece kuadratik balistik ve yerçekimi düşüş önleme çözücüsü.
//! - `InverseKinematics2DoF`: 3B Kartezyen hedeften Pan/Tilt eklem açı ve açısal hızları.
//! - `ActionDiscipline`: Tolerans, salınım sönümlenmesi ve soğuma dolmadan eylem izni vermeyen güvenlik mandalı.

pub mod discipline;
pub mod kinematics;
pub mod latency;
pub mod lead_angle;
pub mod mock;
pub mod types;

pub use discipline::ActionDiscipline;
pub use kinematics::{InverseKinematics2DoF, JointSetpoints};
pub use latency::LatencyCompensator;
pub use lead_angle::{LeadAngleSolver, LeadSolution, GRAVITY};
pub use mock::MockPanTiltRig;
pub use types::{PointingCommand, TargetTrack, TurretFeedback};

#[cfg(test)]
mod tests {
    use super::*;
    use autocore_base::SafetyMode;

    /// Doğrulama 1: 50 ms Gecikmede 20 m/s Hedefin Tam 1.0 m İleri Ekstrapole Edilmesi
    #[test]
    fn test_latency_compensation() {
        let track = TargetTrack {
            position: [100.0, 50.0, 10.0],
            velocity: [20.0, 0.0, 0.0], // 20 m/s X yönünde
            timestamp_us: 1_000_000,
        };

        // 50 ms sonraki eylem anı (50,000 us)
        let action_time_us = 1_050_000;
        let p_real = LatencyCompensator::extrapolate(&track, action_time_us);

        // 20 m/s * 0.05s = 1.0 metre
        assert!((p_real[0] - 101.0).abs() < 1e-4);
        assert!((p_real[1] - 50.0).abs() < 1e-4);
        assert!((p_real[2] - 10.0).abs() < 1e-4);
    }

    /// Doğrulama 2: Mach 3.5 Hipersonik Mühimmat ve Çapraz Hedefe Karşı Balistik Çözüm
    #[test]
    fn test_lead_angle_ballistics() {
        // Hedef: 1000m mesafede, 250 m/s hızla Y yönünde çapraz geçiyor
        let target_pos = [1000.0, 0.0, 200.0];
        let target_vel = [0.0, 250.0, 0.0];

        // Mühimmat: Mach 3.5 (~1190 m/s)
        let vp = 1190.0;

        let sol = LeadAngleSolver::solve(target_pos, target_vel, vp).expect("Çözüm bulunmalı");

        assert!(sol.time_of_flight_s > 0.0);
        let t = sol.time_of_flight_s;

        // Hedefin t anındaki konumu: [1000, 250*t, 200]
        let expected_intercept = [1000.0, 250.0 * t, 200.0];
        for i in 0..3 {
            assert!(
                (sol.intercept_point[i] - expected_intercept[i]).abs() < 1e-3,
                "Kesişim eksen {} hatası < 1mm olmalı",
                i
            );
        }

        // Mühimmatın t sürede aldığı yol hedefe olan mesafeye eşit olmalı: ||p_intercept|| == vp * t
        let dist = (sol.intercept_point[0].powi(2)
            + sol.intercept_point[1].powi(2)
            + sol.intercept_point[2].powi(2))
        .sqrt();
        let proj_dist = vp * t;
        assert!(
            (dist - proj_dist).abs() < 0.01,
            "Mühimmat uçuş mesafesi kesişim mesafesiyle eşleşmeli: dist={}, proj={}",
            dist,
            proj_dist
        );

        // Yerçekimi telafisi: z_aim = z_intercept + 0.5 * g * t^2
        let expected_aim_z = 200.0 + 0.5 * GRAVITY * t * t;
        assert!((sol.aim_point[2] - expected_aim_z).abs() < 1e-3);
    }

    /// Doğrulama 3: Salınım Sönümlenmeden ve Tolerans Dışındayken fire_gate Engeli
    #[test]
    fn test_action_discipline_latch() {
        let mut discipline = ActionDiscipline::new(0.5, 0.05, 50, 100);

        let target_pan = 0.5; // ~28.6 derece
        let target_tilt = 0.2;

        // 1. Durum: Açısal hata büyük (pan = 0.0 iken target = 0.5) -> Fire gate KAPALI
        let gate1 = discipline.evaluate(
            0.0,
            target_tilt,
            target_pan,
            target_tilt,
            0.0,
            0.0,
            SafetyMode::Mission,
            100_000,
        );
        assert!(!gate1, "Açısal hata büyükken ateş açılamaz");

        // 2. Durum: Açılar toleransta ama servo hızla salınıyor (rate = 0.3 rad/s > 0.05) -> KAPALI
        let gate2 = discipline.evaluate(
            target_pan,
            target_tilt,
            target_pan,
            target_tilt,
            0.3,
            0.0,
            SafetyMode::Mission,
            110_000,
        );
        assert!(!gate2, "Salınım hızı yüksekken ateş açılamaz");

        // 3. Durum: Açılar tam, hız sakinleşti ama henüz 50 ms settling time dolmadı -> KAPALI
        let gate3 = discipline.evaluate(
            target_pan,
            target_tilt,
            target_pan,
            target_tilt,
            0.01,
            0.01,
            SafetyMode::Mission,
            120_000, // Başlangıç
        );
        assert!(!gate3, "Settling süresi henüz dolmadı");

        // 4. Durum: 60 ms boyunca sakin kaldı (120_000 + 60_000 = 180_000 us) -> AÇIK!
        let gate4 = discipline.evaluate(
            target_pan,
            target_tilt,
            target_pan,
            target_tilt,
            0.01,
            0.01,
            SafetyMode::Mission,
            180_000,
        );
        assert!(gate4, "Tüm koşullar sağlandı, ateş serbest olmalı!");

        // 5. Durum: Ateş edildi, soğuma süresi (100 ms) dolmadan tekrar basıldı -> KAPALI
        discipline.record_fire(180_000);
        let gate5 = discipline.evaluate(
            target_pan,
            target_tilt,
            target_pan,
            target_tilt,
            0.01,
            0.01,
            SafetyMode::Mission,
            230_000, // 50 ms sonra (100 ms dolmadı)
        );
        assert!(!gate5, "Soğuma süresi bitmeden tekrar ateş açılamaz");

        // 6. Durum: Soğuma süresi dolduktan sonra (180_000 + 110_000 = 290_000 us) ve 50ms sakin kalış
        discipline.evaluate(
            target_pan,
            target_tilt,
            target_pan,
            target_tilt,
            0.01,
            0.01,
            SafetyMode::Mission,
            290_000,
        );
        let gate6 = discipline.evaluate(
            target_pan,
            target_tilt,
            target_pan,
            target_tilt,
            0.01,
            0.01,
            SafetyMode::Mission,
            350_000,
        );
        assert!(gate6, "Soğuma ve settling sonrası ateş tekrar açılmalı");
    }

    /// Doğrulama 4: 2-DoF Ters Kinematik ve Eklem Limitleri
    #[test]
    fn test_inverse_kinematics() {
        let ik = InverseKinematics2DoF::default();

        // 10m ileri, 10m sağ (-Y), 10m yukarı (+Z)
        let aim = [10.0, -10.0, 10.0];
        let vel = [0.0, 0.0, 0.0];

        let sol = ik.solve(aim, vel);

        // Pan: atan2(-10, 10) = -pi/4 (-45 derece)
        assert!((sol.pan_rad - (-std::f32::consts::FRAC_PI_4)).abs() < 1e-4);

        // Tilt: atan2(10, sqrt(200)) = atan2(10, 14.142) ~ 35.26 derece
        assert!(sol.tilt_rad > 0.0);
        assert!(sol.within_limits);

        // Limit dışı elevasyon (aşağı -50 derece, min limit -15 derece)
        let steep_down = [10.0, 0.0, -20.0];
        let sol_down = ik.solve(steep_down, vel);
        assert!(!sol_down.within_limits);
        assert_eq!(sol_down.tilt_rad, -15.0_f32.to_radians());
    }

    /// Doğrulama 5: C-ABI 20-Bayt Sıfır Kopyalama
    #[test]
    fn test_bytemuck_c_abi() {
        let cmd = PointingCommand::new(0.5, 0.2, 0.01, 0.02, true);
        let bytes = cmd.as_bytes();
        assert_eq!(bytes.len(), 20);

        let parsed = PointingCommand::try_from_bytes(bytes).expect("Pod parse başarılı");
        assert_eq!(*parsed, cmd);
        assert!(parsed.is_fire_allowed());
    }
}
