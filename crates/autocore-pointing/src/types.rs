//! autocore-pointing :: C-ABI Yönlendirme ve Angajman Veri Tipleri

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// 20-Bayt C-ABI Uyumlu Yönlendirme ve Atış Disiplini Komutu
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct PointingCommand {
    /// Hedef Azimut (Pan) açısı (Radyan) [-pi, +pi]
    pub target_pan_rad: f32,
    /// Hedef Elevasyon (Tilt) açısı (Radyan) [-pi/2, +pi/2]
    pub target_tilt_rad: f32,
    /// Hedef Azimut açısal hızı (Radyan/saniye)
    pub target_pan_rate_rad_s: f32,
    /// Hedef Elevasyon açısal hızı (Radyan/saniye)
    pub target_tilt_rate_rad_s: f32,
    /// Eylem / Tetik Mandalı: 1 = Ateş Serbest, 0 = Güvenlik Kilidi Aktif
    pub fire_gate: u8,
    /// 4-bayt hizalama için açık dolgu
    pub _pad: [u8; 3],
}

static_assertions::assert_eq_size!(PointingCommand, [u8; 20]);

impl Default for PointingCommand {
    fn default() -> Self {
        Self {
            target_pan_rad: 0.0,
            target_tilt_rad: 0.0,
            target_pan_rate_rad_s: 0.0,
            target_tilt_rate_rad_s: 0.0,
            fire_gate: 0,
            _pad: [0; 3],
        }
    }
}

impl PointingCommand {
    pub fn new(
        target_pan_rad: f32,
        target_tilt_rad: f32,
        target_pan_rate_rad_s: f32,
        target_tilt_rate_rad_s: f32,
        fire_gate: bool,
    ) -> Self {
        Self {
            target_pan_rad,
            target_tilt_rad,
            target_pan_rate_rad_s,
            target_tilt_rate_rad_s,
            fire_gate: if fire_gate { 1 } else { 0 },
            _pad: [0; 3],
        }
    }

    pub fn is_fire_allowed(&self) -> bool {
        self.fire_gate == 1
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<&Self, bytemuck::PodCastError> {
        bytemuck::try_from_bytes(bytes)
    }
}

/// Taret / Gimbal Donanım Geri Bildirimi
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TurretFeedback {
    pub current_pan_rad: f32,
    pub current_tilt_rad: f32,
    pub pan_rate_rad_s: f32,
    pub tilt_rate_rad_s: f32,
    pub timestamp_us: u64,
}

impl Default for TurretFeedback {
    fn default() -> Self {
        Self {
            current_pan_rad: 0.0,
            current_tilt_rad: 0.0,
            pan_rate_rad_s: 0.0,
            tilt_rate_rad_s: 0.0,
            timestamp_us: 0,
        }
    }
}

/// Hedef İz Bilgisi (autocore-vision veya autocore-transform çıkışı)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TargetTrack {
    /// Kartezyen konum [X, Y, Z] metre (Tarete göre)
    pub position: [f32; 3],
    /// Hız vektörü [Vx, Vy, Vz] metre/saniye
    pub velocity: [f32; 3],
    /// Ölçüm zaman damgası (mikrosaniye)
    pub timestamp_us: u64,
}
