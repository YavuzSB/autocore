//! autocore-aerial :: SITL Test İkizi ve Mock Otopilot Aracı (MockMavlinkVehicle)

use crate::mavlink::{MavlinkV2Frame, CRC_EXTRA_SET_POSITION_TARGET_LOCAL_NED};
use crate::types::AutopilotSetpoint;

/// SITL Mock İHA Otopilot Simülatörü (50 Hz Telemetri)
pub struct MockMavlinkVehicle {
    pub position_ned: [f32; 3], // [North, East, Down]
    pub velocity_ned: [f32; 3],
    pub yaw_rad: f32,
    pub altitude_m: f32,
    pub is_armed: bool,
    pub is_rtl: bool,
}

impl Default for MockMavlinkVehicle {
    fn default() -> Self {
        Self::new()
    }
}

impl MockMavlinkVehicle {
    pub fn new() -> Self {
        Self {
            position_ned: [0.0, 0.0, -10.0], // 10 metre irtifada (NED: -Z)
            velocity_ned: [0.0, 0.0, 0.0],
            yaw_rad: 0.0,
            altitude_m: 10.0,
            is_armed: true,
            is_rtl: false,
        }
    }

    /// Bir zaman adımı simülasyon işletir (dt_s = 0.02s / 50 Hz)
    pub fn step(&mut self, setpoint: &AutopilotSetpoint, dt_s: f32) {
        if setpoint.is_rtl() {
            self.is_rtl = true;
            // RTL: Kalkış noktasına (0,0) dön ve in
            let dist = (self.position_ned[0].powi(2) + self.position_ned[1].powi(2)).sqrt();
            if dist > 1.0 {
                let dir_n = -self.position_ned[0] / dist;
                let dir_e = -self.position_ned[1] / dist;
                self.velocity_ned = [dir_n * 5.0, dir_e * 5.0, 0.0];
            } else {
                // Alçalış
                self.velocity_ned = [0.0, 0.0, 1.0]; // +Z = aşağı
            }
        } else {
            self.velocity_ned = [setpoint.ned_vx, setpoint.ned_vy, setpoint.ned_vz];
            self.yaw_rad += setpoint.yaw_rate_rad_s * dt_s;
        }

        self.position_ned[0] += self.velocity_ned[0] * dt_s;
        self.position_ned[1] += self.velocity_ned[1] * dt_s;
        self.position_ned[2] += self.velocity_ned[2] * dt_s;
        self.altitude_m = -self.position_ned[2];
    }

    /// MAVLink v2 SET_POSITION_TARGET_LOCAL_NED bayt paketini işler
    pub fn handle_mavlink_packet(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        let frame = MavlinkV2Frame::decode(bytes, CRC_EXTRA_SET_POSITION_TARGET_LOCAL_NED)?;
        if let Some(vel) = frame.extract_velocity_ned() {
            self.velocity_ned = vel;
            Ok(())
        } else {
            Err("Desteklenmeyen veya geçersiz paket")
        }
    }
}
