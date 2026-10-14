//! autocore-marine :: C-ABI İnsansız Deniz Aracı (İDA) İtici Sinyal Tipleri

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// 16-Bayt C-ABI Uyumlu İtici ve Dümen Aktüatör Sinyali
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct ThrusterSignals {
    /// İskele (Sol) motor itki yüzdesi [-1.0, +1.0]
    pub port_pct: f32,
    /// Sağ motor itki yüzdesi [-1.0, +1.0]
    pub starboard_pct: f32,
    /// Dümen açısı (Radyan) [-pi/4, +pi/4] (Varsa)
    pub rudder_angle_rad: f32,
    /// 8-bayt hizalama dolgusu
    pub _pad: u32,
}

static_assertions::assert_eq_size!(ThrusterSignals, [u8; 16]);

impl Default for ThrusterSignals {
    fn default() -> Self {
        Self {
            port_pct: 0.0,
            starboard_pct: 0.0,
            rudder_angle_rad: 0.0,
            _pad: 0,
        }
    }
}

impl ThrusterSignals {
    pub fn new(port_pct: f32, starboard_pct: f32, rudder_angle_rad: f32) -> Self {
        Self {
            port_pct: port_pct.clamp(-1.0, 1.0),
            starboard_pct: starboard_pct.clamp(-1.0, 1.0),
            rudder_angle_rad,
            _pad: 0,
        }
    }

    pub fn neutral() -> Self {
        Self::default()
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<&Self, bytemuck::PodCastError> {
        bytemuck::try_from_bytes(bytes)
    }
}

/// Hidrodinamik Kuvvet ve Moment Talebi (3-DoF Düzlem)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MarineForceRequest {
    /// İleri/Geri itki kuvveti (Newton)
    pub fx_n: f32,
    /// Yanal kuvvet (Newton, omni/yan iterli araçlar için)
    pub fy_n: f32,
    /// Dikey eksen etrafında dönme torku (Newton-metre)
    pub torque_z_nm: f32,
}

impl Default for MarineForceRequest {
    fn default() -> Self {
        Self {
            fx_n: 0.0,
            fy_n: 0.0,
            torque_z_nm: 0.0,
        }
    }
}
