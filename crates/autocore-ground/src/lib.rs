//! # autocore-ground
//!
//! Zemin Seyrüsefer, Çoklu Şasi Kinematiği, 2D Maliyet Haritası ve Yol Takip Çekirdeği.
//!
//! - `kinematics`: Diferansiyel, Ackermann ve Mecanum çoklu şasi hız ayrıştırıcıları.
//! - `costmap`: BFS üstel sönümlü 2D yerel maliyet haritası (Costmap2D).
//! - `avoidance`: Dinamik Pencere Yaklaşımı (DWA) ve Yapay Potansiyel Alanlar (APF).
//! - `path_tracker`: Pure Pursuit ve Stanley yol takip kontrolcüleri.

pub mod avoidance;
pub mod costmap;
pub mod kinematics;
pub mod path_tracker;

pub use avoidance::{ArtificialPotentialField, DynamicWindowApproach};
pub use costmap::{LocalCostmap2D, COST_FREE, COST_INFLATED_MAX, COST_LETHAL, COST_UNKNOWN};
pub use kinematics::{
    AckermannCommand, AckermannKinematics, DifferentialDriveKinematics, IdaThrusterMatrix,
    MecanumKinematics, MecanumWheelSpeeds, ThrusterAllocation, WheelSpeeds,
};
pub use path_tracker::{PurePursuitTracker, StanleyTracker};

#[cfg(test)]
mod tests {
    use super::*;

    /// Doğrulama: Pure Pursuit Rota Takipçisi Açısal Hız Üretimi
    #[test]
    fn test_pure_pursuit_tracking() {
        let tracker = PurePursuitTracker::new(2.0, 2.0);

        // Hedef 2m ileri, 1m solda (+Y): sola dönüş (omega > 0) üretmeli
        let omega = tracker.compute_angular_velocity([2.0, 1.0], 1.5);
        assert!(omega > 0.0);

        // Hedef tam ileride [2.0, 0.0]: omega = 0.0 olmalı
        let omega_straight = tracker.compute_angular_velocity([2.0, 0.0], 1.5);
        assert_eq!(omega_straight, 0.0);
    }

    /// Doğrulama: Stanley Rota Takipçisi Direksiyon Açısı Üretimi
    #[test]
    fn test_stanley_tracking() {
        let tracker = StanleyTracker::new(0.8, 35.0);

        // Yol düz kuzey, araç çizgiden 0.5m sağda (+e_fa) ve sola 10 derece bakıyor:
        // Düzeltici direksiyon açısı sola doğru (negatif/pozitif tanımlı) açılmalı
        let steer = tracker.compute_steer_angle(0.0, 0.5, 2.0);
        assert!(steer > 0.0);
    }
}
