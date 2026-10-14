//! autocore-arbiter :: C-ABI Komut ve Emniyet Hakemi Veri Tipleri

use autocore_base::{KinematicCommand, SafetyMode};
use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// Komut Öncelik Seviyesi (Strict Priority Tiers)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PriorityTier {
    /// Tier 1: Mutlak Emniyet & Veto (E-Stop, SafeLock, Watchdog)
    Tier1Safety = 1,
    /// Tier 2: Reaktif Güvenlik (Engel Kaçınma DWA/APF, Stall/AoA Guard)
    Tier2Reactive = 2,
    /// Tier 3: Kullanıcı Görev Katmanı (Mission Logic)
    Tier3Mission = 3,
}

impl PriorityTier {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(Self::Tier1Safety),
            2 => Some(Self::Tier2Reactive),
            3 => Some(Self::Tier3Mission),
            _ => None,
        }
    }
}

/// Hakem Devreye Girme / Ezme Gerekçesi
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverrideReason {
    /// Normal çalışma, ezme yok
    None = 0,
    /// Donanımsal veya yazılımsal E-Stop tetiklendi
    EStop = 1,
    /// Reaktif engel kaçınma devrede (Costmap/DWA/APF)
    Obstacle = 2,
    /// 500ms Watchdog zaman aşımı veya SafeLock
    WatchdogSafeLock = 3,
    /// Sanal Geofence sınır ihlali
    GeofenceInhibit = 4,
    /// Sistem pasif veya ilklendirme aşamasında (Disarmed/Booting)
    SystemDisarmed = 5,
}

impl OverrideReason {
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => Self::EStop,
            2 => Self::Obstacle,
            3 => Self::WatchdogSafeLock,
            4 => Self::GeofenceInhibit,
            5 => Self::SystemDisarmed,
            _ => Self::None,
        }
    }
}

/// 40-Bayt C-ABI Uyumlu Hakem Tarafından Onaylanmış Komut
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct ArbitratedCommand {
    /// Aktüatöre iletilecek kinematik hız komutu (32 Bayt)
    pub command: KinematicCommand,
    /// Aktif öncelik seviyesi (1: Tier 1, 2: Tier 2, 3: Tier 3)
    pub active_tier: u8,
    /// Ezme/öncelik gerekçesi (OverrideReason)
    pub override_reason: u8,
    /// Gelecek kullanım için rezerve alan
    pub reserved: u16,
    /// 8-bayt hizalama için açık dolgu
    pub _pad: u32,
}

static_assertions::assert_eq_size!(ArbitratedCommand, [u8; 40]);

impl Default for ArbitratedCommand {
    fn default() -> Self {
        Self {
            command: KinematicCommand::default(),
            active_tier: PriorityTier::Tier1Safety as u8,
            override_reason: OverrideReason::SystemDisarmed as u8,
            reserved: 0,
            _pad: 0,
        }
    }
}

impl ArbitratedCommand {
    pub fn new(
        command: KinematicCommand,
        active_tier: PriorityTier,
        override_reason: OverrideReason,
    ) -> Self {
        Self {
            command,
            active_tier: active_tier as u8,
            override_reason: override_reason as u8,
            reserved: 0,
            _pad: 0,
        }
    }

    /// Tamamen sıfırlanmış Tier 1 E-Stop fren komutu üretir
    pub fn neutral_veto(reason: OverrideReason, timestamp_us: u64) -> Self {
        Self {
            command: KinematicCommand::neutral(timestamp_us),
            active_tier: PriorityTier::Tier1Safety as u8,
            override_reason: reason as u8,
            reserved: 0,
            _pad: 0,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<&Self, bytemuck::PodCastError> {
        bytemuck::try_from_bytes(bytes)
    }
}

/// Tier 1 Emniyet Durum Girdisi
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tier1SafetyStatus {
    pub safety_mode: SafetyMode,
    pub estop_active: bool,
    pub geofence_breach: bool,
    pub watchdog_timeout: bool,
}

impl Default for Tier1SafetyStatus {
    fn default() -> Self {
        Self {
            safety_mode: SafetyMode::Disarmed,
            estop_active: false,
            geofence_breach: false,
            watchdog_timeout: false,
        }
    }
}

impl Tier1SafetyStatus {
    pub fn safe_operating() -> Self {
        Self {
            safety_mode: SafetyMode::Mission,
            estop_active: false,
            geofence_breach: false,
            watchdog_timeout: false,
        }
    }
}
