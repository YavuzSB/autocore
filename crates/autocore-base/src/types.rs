//! autocore-base :: Çekirdek Veri Tipleri ve C-ABI Uyumlu Veri Yapıları

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// Sistem Emniyet Durumu (7 Durumlu FSM Değerleri)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SafetyMode {
    /// Sistem yeni enerjilendi, sensörler ve IPC ilklendiriliyor
    Booting = 0,
    /// Sistem açık, hazır ancak motorlar ve aktüatörler pasif (Disarmed)
    Disarmed = 1,
    /// Motorlar devrede, manuel veya bekleme modunda (Armed)
    Armed = 2,
    /// Tam otonom görev icra ediliyor (Mission)
    Mission = 3,
    /// Bağlantı kopması veya sensör kaybı; güvenli iniş/dönüş modu (Failsafe)
    Failsafe = 4,
    /// Kritik kilit: 500ms Watchdog zaman aşımı, E-Stop veya SegFault (SafeLock)
    SafeLock = 5,
    /// Fiziksel pilot veya emniyet hakemi kumandası devrede (ManualOverride)
    ManualOverride = 6,
}

impl SafetyMode {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Booting),
            1 => Some(Self::Disarmed),
            2 => Some(Self::Armed),
            3 => Some(Self::Mission),
            4 => Some(Self::Failsafe),
            5 => Some(Self::SafeLock),
            6 => Some(Self::ManualOverride),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Booting => "BOOTING",
            Self::Disarmed => "DISARMED",
            Self::Armed => "ARMED",
            Self::Mission => "MISSION",
            Self::Failsafe => "FAILSAFE",
            Self::SafeLock => "SAFELOCK",
            Self::ManualOverride => "MANUAL_OVERRIDE",
        }
    }
}

/// Standart Kinematik Hız Komutu Sözleşmesi (32 Bayt, C-ABI)
///
/// V = [Vx, Vy, Vz, RollRate, PitchRate, YawRate]^T ∈ R^6
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct KinematicCommand {
    /// Doğrusal hızlar [Vx, Vy, Vz] (metre/saniye)
    pub linear: [f32; 3],
    /// Açısal hızlar [RollRate, PitchRate, YawRate] (radyan/saniye)
    pub angular: [f32; 3],
    /// Mikrosaniye cinsinden gönderim zaman damgası (UNIX epoch veya monotonic)
    pub timestamp_us: u64,
}

impl Default for KinematicCommand {
    fn default() -> Self {
        Self {
            linear: [0.0; 3],
            angular: [0.0; 3],
            timestamp_us: 0,
        }
    }
}

impl KinematicCommand {
    pub fn new(linear: [f32; 3], angular: [f32; 3], timestamp_us: u64) -> Self {
        Self {
            linear,
            angular,
            timestamp_us,
        }
    }

    /// Tamamen sıfırlanmış fren/durma komutu üretir
    pub fn neutral(timestamp_us: u64) -> Self {
        Self {
            linear: [0.0; 3],
            angular: [0.0; 3],
            timestamp_us,
        }
    }

    /// Ham bayt dizisine dönüştürür (Sıfır kopyalama)
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    /// Ham bayt dizisinden okur (Sıfır kopyalama)
    pub fn try_from_bytes(bytes: &[u8]) -> Result<&Self, bytemuck::PodCastError> {
        bytemuck::try_from_bytes(bytes)
    }
}

/// 6-DoF Kinematik Durum Kestirim Vektörü (64 Bayt, C-ABI)
///
/// Uzaysal seyrüsefer yapan (İHA, İDA, İKA, Füze) araçların durum vektörü.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct StateVector6DoF {
    /// Kartezyen konum [X, Y, Z] (Metre - NED veya yerel koordinat)
    pub position: [f32; 3],
    /// Doğrusal hızlar [Vx, Vy, Vz] (Metre/saniye)
    pub velocity: [f32; 3],
    /// Yönelim Kuaterniyonu [qw, qx, qy, qz] (Norm = 1.0)
    pub quaternion: [f32; 4],
    /// Açısal hızlar [P, Q, R] (Radyan/saniye)
    pub angular_rates: [f32; 3],
    /// 64-bayt hizalamayı garantileyen açık dolgu
    pub _pad: [u8; 4],
    /// Mikrosaniye zaman damgası
    pub timestamp_us: u64,
}

impl Default for StateVector6DoF {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            quaternion: [1.0, 0.0, 0.0, 0.0], // Birim kuaterniyon
            angular_rates: [0.0; 3],
            _pad: [0; 4],
            timestamp_us: 0,
        }
    }
}

impl StateVector6DoF {
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<&Self, bytemuck::PodCastError> {
        bytemuck::try_from_bytes(bytes)
    }
}

/// Aktüatör ve Donanım Geri Besleme Telemetrisi (72 Bayt, C-ABI)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct ActuatorFeedback {
    /// Gerçekleşen doğrusal hızlar [Vx, Vy, Vz] (m/s)
    pub actual_linear: [f32; 3],
    /// Gerçekleşen açısal hızlar [Roll, Pitch, Yaw] (rad/s)
    pub actual_angular: [f32; 3],
    /// Motor akımları [M1, M2, M3, M4] (Amper)
    pub motor_currents: [f32; 4],
    /// Motor/Sürücü sıcaklıkları [S1, S2, S3, S4] (Santigrat)
    pub temperatures: [f32; 4],
    /// Donanımsal durum bayrakları (bitmask: E-Stop, Limit Switch, Aşırı Akım vb.)
    pub status_flags: u32,
    /// 8-bayt hizalama için açık dolgu
    pub _pad: u32,
    /// Mikrosaniye zaman damgası
    pub timestamp_us: u64,
}

impl Default for ActuatorFeedback {
    fn default() -> Self {
        Self {
            actual_linear: [0.0; 3],
            actual_angular: [0.0; 3],
            motor_currents: [0.0; 4],
            temperatures: [0.0; 4],
            status_flags: 0,
            _pad: 0,
            timestamp_us: 0,
        }
    }
}

impl ActuatorFeedback {
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<&Self, bytemuck::PodCastError> {
        bytemuck::try_from_bytes(bytes)
    }
}
