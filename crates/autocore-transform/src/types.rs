//! autocore-transform :: C-ABI Uyumlu Veri Yapıları ve IPC Sözleşmeleri

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// 48-Bayt C-ABI Uyumlu Yerel Durum Pozu (Zero-Copy Paylaşılan Bellek)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct LocalPose {
    /// Yerel Kartezyen konum [X, Y, Z] (Metre - NED/Yerel referans)
    pub position: [f32; 3],
    /// Birim kuaterniyon [qw, qx, qy, qz]
    pub orientation: [f32; 4],
    /// Doğrusal hızlar [Vx, Vy, Vz] (Metre/saniye)
    pub velocity: [f32; 3],
    /// Mikrosaniye zaman damgası
    pub timestamp_us: u64,
}

static_assertions::assert_eq_size!(LocalPose, [u8; 48]);

impl Default for LocalPose {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            orientation: [1.0, 0.0, 0.0, 0.0],
            velocity: [0.0; 3],
            timestamp_us: 0,
        }
    }
}

impl LocalPose {
    pub fn new(
        position: [f32; 3],
        orientation: [f32; 4],
        velocity: [f32; 3],
        timestamp_us: u64,
    ) -> Self {
        Self {
            position,
            orientation,
            velocity,
            timestamp_us,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<&Self, bytemuck::PodCastError> {
        bytemuck::try_from_bytes(bytes)
    }
}

/// 40-Bayt C-ABI Ham GNSS / GPS Verisi
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct GnssRaw {
    /// Enlem (Latitude) - derece
    pub lat_deg: f64,
    /// Boylam (Longitude) - derece
    pub lon_deg: f64,
    /// İrtifa (Altitude) - metre
    pub alt_m: f64,
    /// Yatay Geometrik Seyrelme (HDOP)
    pub hdop: f32,
    /// 8-bayt hizalama için açık dolgu
    pub _pad: u32,
    /// Mikrosaniye zaman damgası
    pub timestamp_us: u64,
}

static_assertions::assert_eq_size!(GnssRaw, [u8; 40]);

impl Default for GnssRaw {
    fn default() -> Self {
        Self {
            lat_deg: 0.0,
            lon_deg: 0.0,
            alt_m: 0.0,
            hdop: 1.0,
            _pad: 0,
            timestamp_us: 0,
        }
    }
}

impl GnssRaw {
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<&Self, bytemuck::PodCastError> {
        bytemuck::try_from_bytes(bytes)
    }
}

/// 72-Bayt C-ABI Zaman Damgalı Çerçeve Dönüşümü (TfStamped)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct TfStamped {
    /// Ebeveyn koordinat çerçevesi (ASCII, sıfır sonlandırmalı)
    pub parent_frame: [u8; 16],
    /// Çocuk koordinat çerçevesi (ASCII, sıfır sonlandırmalı)
    pub child_frame: [u8; 16],
    /// Öteleme vektörü [x, y, z] metre
    pub translation: [f32; 3],
    /// Dönme kuaterniyonu [qw, qx, qy, qz]
    pub rotation: [f32; 4],
    /// 8-bayt hizalama dolgusu
    pub _pad: u32,
    /// Mikrosaniye zaman damgası
    pub timestamp_us: u64,
}

static_assertions::assert_eq_size!(TfStamped, [u8; 72]);

impl Default for TfStamped {
    fn default() -> Self {
        Self {
            parent_frame: [0; 16],
            child_frame: [0; 16],
            translation: [0.0; 3],
            rotation: [1.0, 0.0, 0.0, 0.0],
            _pad: 0,
            timestamp_us: 0,
        }
    }
}

impl TfStamped {
    pub fn new(
        parent: &str,
        child: &str,
        translation: [f32; 3],
        rotation: [f32; 4],
        timestamp_us: u64,
    ) -> Self {
        let mut parent_bytes = [0u8; 16];
        let p_slice = parent.as_bytes();
        let p_len = p_slice.len().min(15);
        parent_bytes[..p_len].copy_from_slice(&p_slice[..p_len]);

        let mut child_bytes = [0u8; 16];
        let c_slice = child.as_bytes();
        let c_len = c_slice.len().min(15);
        child_bytes[..c_len].copy_from_slice(&c_slice[..c_len]);

        Self {
            parent_frame: parent_bytes,
            child_frame: child_bytes,
            translation,
            rotation,
            _pad: 0,
            timestamp_us,
        }
    }

    pub fn parent_str(&self) -> &str {
        let len = self
            .parent_frame
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(16);
        std::str::from_utf8(&self.parent_frame[..len]).unwrap_or("")
    }

    pub fn child_str(&self) -> &str {
        let len = self.child_frame.iter().position(|&b| b == 0).unwrap_or(16);
        std::str::from_utf8(&self.child_frame[..len]).unwrap_or("")
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<&Self, bytemuck::PodCastError> {
        bytemuck::try_from_bytes(bytes)
    }
}
