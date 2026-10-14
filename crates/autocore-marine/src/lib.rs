//! # autocore-marine
//!
//! Hidrodinamik Seyrüsefer, İtki Dağıtımı ve Akıntı Dengeleme Çekirdeği.
//!
//! - `ThrusterAllocationMatrix`: Genel [Fx, Fy, Mz] talebini motor itkilerine dağıtır.
//! - `DriftCompensator`: Akıntı ve rüzgar kaynaklı sürüklenmeyi yengeç açısıyla kompanse eder.
//! - `MarineNavigator`: Kavitasyon önleyici itki profili ve alabora koruması.

pub mod allocation;
pub mod drift;
pub mod mock;
pub mod navigator;
pub mod types;

pub use allocation::ThrusterAllocationMatrix;
pub use drift::DriftCompensator;
pub use mock::MockMarineSimulator;
pub use navigator::MarineNavigator;
pub use types::{MarineForceRequest, ThrusterSignals};

#[cfg(test)]
mod tests {
    use super::*;

    /// Doğrulama 1: Fx = 100 N, Mz = 0 için iskele ve sağ motorlara eşit itki dağıtılması
    #[test]
    fn test_thruster_allocation_matrix() {
        let alloc = ThrusterAllocationMatrix::new(0.8, 100.0); // max 100 N per motor

        // Sadece ileri itki talebi: 100 N toplam (50 N sol, 50 N sağ beklenir = %50)
        let req_fwd = MarineForceRequest {
            fx_n: 100.0,
            fy_n: 0.0,
            torque_z_nm: 0.0,
        };

        let sig_fwd = alloc.allocate_differential(&req_fwd);
        assert!((sig_fwd.port_pct - 0.5).abs() < 1e-4);
        assert!((sig_fwd.starboard_pct - 0.5).abs() < 1e-4);

        // Saf dönme torku talebi: Mz = 40 Nm (beam = 0.8m -> half_beam = 0.4m)
        // delta_T = 40 / (2 * 0.4) = 50 N
        // port = -50 N (% -50), stbd = +50 N (% +50)
        let req_turn = MarineForceRequest {
            fx_n: 0.0,
            fy_n: 0.0,
            torque_z_nm: 40.0,
        };

        let sig_turn = alloc.allocate_differential(&req_turn);
        assert!((sig_turn.port_pct - (-0.5)).abs() < 1e-4);
        assert!((sig_turn.starboard_pct - 0.5).abs() < 1e-4);
    }

    /// Doğrulama 2: Yanal akıntıda pruvanın akıntıya doğru açı alarak yanal hatanın sıfırlanması
    #[test]
    fn test_drift_compensation_crabbing() {
        let mut compensator = DriftCompensator::default();

        let path_heading = 0.0; // Kuzey rotası
        let crosstrack_error = 2.0; // 2m sağa sürüklenmiş
        let estimated_drift = 1.0; // 1 m/s akıntı
        let forward_speed = 3.0; // 3 m/s hız
        let dt_s = 0.1;

        let (heading_cmd, crab_angle) = compensator.compute_compensated_heading(
            path_heading,
            crosstrack_error,
            estimated_drift,
            forward_speed,
            dt_s,
        );

        // Yengeç açısı pozitif olmalı (akıntı yönüne karşı)
        assert!(crab_angle > 0.0);
        // Kompanse edilmiş rota pruvayı akıntıya doğru kırmalı (heading_cmd < 0 veya sola)
        assert!(heading_cmd < path_heading);
    }

    /// Doğrulama 3: Ani tork değişimlerinde motor ivmesinin sınırlandırılması (Kavitasyon Koruması)
    #[test]
    fn test_cavitation_rate_limit() {
        let mut nav = MarineNavigator::new(1.0, 1.5); // max %100/saniye değişim

        // %0'dan aniden %100 tam gaza basış isteği
        let target = ThrusterSignals::new(1.0, 1.0, 0.0);

        // 0.2 saniye sonra: max delta = 1.0 * 0.2 = %20 (0.2)
        let filtered1 = nav.filter_cavitation(&target, 0.2);
        assert!((filtered1.port_pct - 0.2).abs() < 1e-4);
        assert!((filtered1.starboard_pct - 0.2).abs() < 1e-4);

        // Bir 0.3 saniye sonra: toplam 0.2 + 0.3 = 0.5
        let filtered2 = nav.filter_cavitation(&target, 0.3);
        assert!((filtered2.port_pct - 0.5).abs() < 1e-4);
    }

    /// Doğrulama 4: C-ABI 16-Bayt Sıfır Kopyalama
    #[test]
    fn test_bytemuck_c_abi() {
        let sig = ThrusterSignals::new(0.75, -0.25, 0.1);
        let bytes = sig.as_bytes();
        assert_eq!(bytes.len(), 16);

        let parsed = ThrusterSignals::try_from_bytes(bytes).expect("Pod parse başarılı");
        assert_eq!(*parsed, sig);
    }
}
