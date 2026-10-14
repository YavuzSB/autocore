//! autocore-aerial :: C-ABI Uyumlu Uçuş ve Otopilot Setpoint Veri Tipleri

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// 28-Bayt C-ABI Uyumlu Otopilot Hedef Hız ve İvme Paketi
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct AutopilotSetpoint {
    /// NED Yerel Kuzey Hızı (m/s)
    pub ned_vx: f32,
    /// NED Yerel Doğu Hızı (m/s)
    pub ned_vy: f32,
    /// NED Yerel Aşağı Hızı (m/s, negatif = tırmanma)
    pub ned_vz: f32,
    /// Hedef Yaw Açısal Hızı (rad/s)
    pub yaw_rate_rad_s: f32,
    /// Z-Ekseni İvme Komutu (m/s^2)
    pub z_accel_m_s2: f32,
    /// Otopilot mod bayrakları (Bit 0: Armed, Bit 1: Offboard, Bit 2: RTL, Bit 3: Land)
    pub mode_flags: u32,
    /// 8-bayt hizalama dolgusu
    pub _pad: u32,
}

static_assertions::assert_eq_size!(AutopilotSetpoint, [u8; 28]);

impl Default for AutopilotSetpoint {
    fn default() -> Self {
        Self {
            ned_vx: 0.0,
            ned_vy: 0.0,
            ned_vz: 0.0,
            yaw_rate_rad_s: 0.0,
            z_accel_m_s2: 0.0,
            mode_flags: 0,
            _pad: 0,
        }
    }
}

impl AutopilotSetpoint {
    pub const FLAG_ARMED: u32 = 1 << 0;
    pub const FLAG_OFFBOARD: u32 = 1 << 1;
    pub const FLAG_RTL: u32 = 1 << 2;
    pub const FLAG_LAND: u32 = 1 << 3;

    pub fn new(
        ned_vx: f32,
        ned_vy: f32,
        ned_vz: f32,
        yaw_rate_rad_s: f32,
        z_accel_m_s2: f32,
        mode_flags: u32,
    ) -> Self {
        Self {
            ned_vx,
            ned_vy,
            ned_vz,
            yaw_rate_rad_s,
            z_accel_m_s2,
            mode_flags,
            _pad: 0,
        }
    }

    pub fn rtl() -> Self {
        Self {
            ned_vx: 0.0,
            ned_vy: 0.0,
            ned_vz: 0.0,
            yaw_rate_rad_s: 0.0,
            z_accel_m_s2: 0.0,
            mode_flags: Self::FLAG_ARMED | Self::FLAG_RTL,
            _pad: 0,
        }
    }

    pub fn is_rtl(&self) -> bool {
        (self.mode_flags & Self::FLAG_RTL) != 0
    }

    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<&Self, bytemuck::PodCastError> {
        bytemuck::try_from_bytes(bytes)
    }
}
