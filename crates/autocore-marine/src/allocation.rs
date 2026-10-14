//! autocore-marine :: Hidrodinamik İtki Dağıtım Matrisi (ThrusterAllocationMatrix)
//!
//! Genel kuvvet/tork talebini [Fx, Fy, Mz] çoklu pervane veya çift motorlu İDA'ya
//! Moore-Penrose genelleştirilmiş matris tersiyle dağıtır.

use crate::types::{MarineForceRequest, ThrusterSignals};

/// Çift Motorlu Diferansiyel İDA İtki Tahsis Motoru
#[derive(Debug, Clone, Copy)]
pub struct ThrusterAllocationMatrix {
    /// İki motor ekseni arası genişlik mesafesi (metre)
    pub beam_width_m: f32,
    /// Motor başına maksimum itki kuvveti (Newton)
    pub max_thrust_per_motor_n: f32,
}

impl Default for ThrusterAllocationMatrix {
    fn default() -> Self {
        Self {
            beam_width_m: 0.8, // 80 cm genişlik
            max_thrust_per_motor_n: 150.0, // 150 N (~15 kgf)
        }
    }
}

impl ThrusterAllocationMatrix {
    pub fn new(beam_width_m: f32, max_thrust_per_motor_n: f32) -> Self {
        Self {
            beam_width_m: beam_width_m.max(0.1),
            max_thrust_per_motor_n: max_thrust_per_motor_n.max(1.0),
        }
    }

    /// Çift motorlu İDA için [Fx, Mz] talebini iskele ve sağ motor itkilerine çözer:
    /// u_port = Fx/2 - Mz/(2*d)
    /// u_starboard = Fx/2 + Mz/(2*d)
    pub fn allocate_differential(&self, req: &MarineForceRequest) -> ThrusterSignals {
        let half_beam = self.beam_width_m * 0.5;

        // B Matrisi: [1, 1; -d, d]
        // Çözüm:
        let thrust_port_n = (req.fx_n * 0.5) - (req.torque_z_nm / (2.0 * half_beam));
        let thrust_stbd_n = (req.fx_n * 0.5) + (req.torque_z_nm / (2.0 * half_beam));

        // [-1.0, 1.0] aralığına normalize et
        let port_pct = (thrust_port_n / self.max_thrust_per_motor_n).clamp(-1.0, 1.0);
        let stbd_pct = (thrust_stbd_n / self.max_thrust_per_motor_n).clamp(-1.0, 1.0);

        ThrusterSignals::new(port_pct, stbd_pct, 0.0)
    }

    /// 4 İticili Vektörlü / Omni Su Altı & Su Üstü Matris Dağıtımı
    pub fn allocate_4thruster_vectored(&self, fx: f32, fy: f32, mz: f32) -> [f32; 4] {
        // 45 derece açılı 4 itici geometrisi (X-konfigürasyonu):
        // B matrisi üzerinden psödo-ters:
        let c = std::f32::consts::FRAC_1_SQRT_2;
        let d = self.beam_width_m * 0.5;

        let t1 = (fx * c - fy * c - mz / (4.0 * d)) / self.max_thrust_per_motor_n;
        let t2 = (fx * c + fy * c + mz / (4.0 * d)) / self.max_thrust_per_motor_n;
        let t3 = (fx * c + fy * c - mz / (4.0 * d)) / self.max_thrust_per_motor_n;
        let t4 = (fx * c - fy * c + mz / (4.0 * d)) / self.max_thrust_per_motor_n;

        [
            t1.clamp(-1.0, 1.0),
            t2.clamp(-1.0, 1.0),
            t3.clamp(-1.0, 1.0),
            t4.clamp(-1.0, 1.0),
        ]
    }
}
