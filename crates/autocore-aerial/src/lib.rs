//! # autocore-aerial
//!
//! Uçuş Dinamiği, Otopilot Köprüsü ve 3B Hava Güdümü Çekirdeği.
//!
//! - `MavlinkBridge`: PX4/ArduPilot için MAVLink v2 paket kodlama/çözme.
//! - `L1Guidance`: Park et al. doğrusal olmayan L1 yol güdümü (crosstrack düzeltmesi).
//! - `FlightEnvelopeGuard`: Stall önleme, bank açısı limiti (<=45 deg) ve tırmanma koruyucusu.
//! - `AirGeofence`: 3B irtifa penceresi ve poligon sınır ihlalinde otomatik RTL.
//! - `VtolTransitionController`: Hover -> Transition -> Cruise aerodinamik itki mikseri.

pub mod envelope;
pub mod geofence;
pub mod l1_guidance;
pub mod mavlink;
pub mod mock;
pub mod types;
pub mod vtol;

pub use envelope::{FlightEnvelopeGuard, GuardedFlightCommand};
pub use geofence::{AirGeofence, GeofenceStatus};
pub use l1_guidance::{L1Guidance, L1Output};
pub use mavlink::{
    MavlinkV2Frame, CRC_EXTRA_COMMAND_LONG, CRC_EXTRA_HEARTBEAT,
    CRC_EXTRA_SET_POSITION_TARGET_LOCAL_NED,
};
pub use mock::MockMavlinkVehicle;
pub use types::AutopilotSetpoint;
pub use vtol::{VtolActuatorMixing, VtolState, VtolTransitionController};

#[cfg(test)]
mod tests {
    use super::*;

    /// Doğrulama 1: Rüzgar altında çizgiden 5 m sapan uçağın a_s ile merkeze dönmesi
    #[test]
    fn test_l1_crosstrack_correction() {
        let l1 = L1Guidance::default();

        // Yol çizgisi: Kuzey yönünde düz hat (Start: [0, 0], End: [1000, 0])
        let seg_start = [0.0, 0.0];
        let seg_end = [1000.0, 0.0];

        // Uçak: Çizginin 5 metre doğusunda (sağında) seyrediyor [100.0, 5.0], kuzeye bakıyor (heading = 0)
        let current_pos = [100.0, 5.0];
        let ground_speed = 20.0; // 20 m/s
        let heading_rad = 0.0;

        let out = l1.compute(current_pos, ground_speed, heading_rad, seg_start, seg_end);

        // Yanal sapma 5m olmalı
        assert!((out.crosstrack_error_m - 5.0).abs() < 1e-3);

        // Çizginin sağında olduğu için aracı sola (negatif ivme/yaw rate) çekmeli!
        assert!(
            out.lateral_accel_m_s2 < 0.0,
            "Sola dönüş ivmesi üretilmeli: {}",
            out.lateral_accel_m_s2
        );
        assert!(out.yaw_rate_rad_s < 0.0);
    }

    /// Doğrulama 2: 60 derece yatış komutunun 45 dereceye kırpılması (Bank limit)
    #[test]
    fn test_envelope_bank_limit() {
        let guard = FlightEnvelopeGuard::default();

        // 60 derece roll komutu (+pi/3 ~ 1.047 rad)
        let raw_roll = 60.0_f32.to_radians();
        let raw_pitch = 0.0;
        let raw_vz = 0.0;
        let airspeed = 25.0; // Güvenli hız

        let cmd = guard.guard(raw_roll, raw_pitch, raw_vz, airspeed);

        assert!(cmd.is_bank_clamped);
        let max_bank = 45.0_f32.to_radians();
        assert!(
            (cmd.roll_cmd_rad - max_bank).abs() < 1e-4,
            "Roll 45 dereceye kırpılmalı: {} rad",
            cmd.roll_cmd_rad
        );

        // Stall koruma testi: Hava hızı 8 m/s (< 12 m/s stall speed)
        let stall_cmd = guard.guard(0.0, 10.0_f32.to_radians(), 0.0, 8.0);
        assert!(stall_cmd.is_stall_warning);
        assert!(stall_cmd.pitch_cmd_rad <= -5.0_f32.to_radians()); // Burnu aşağı ezmeli
    }

    /// Doğrulama 3: 120m irtifa tavanı aşıldığında RTL komutunun tetiklenmesi
    #[test]
    fn test_air_geofence_rtl_trigger() {
        let geofence = AirGeofence::default(); // max 120m

        let normal_cmd = AutopilotSetpoint::new(10.0, 0.0, 0.0, 0.0, 0.0, AutopilotSetpoint::FLAG_ARMED);

        // 1. Güvenli irtifa: 50 metre
        let (out_safe, status_safe) = geofence.enforce(normal_cmd, [0.0, 0.0], 50.0);
        assert_eq!(status_safe, GeofenceStatus::Safe);
        assert!(!out_safe.is_rtl());
        assert_eq!(out_safe.ned_vx, 10.0);

        // 2. İrtifa ihlali: 125 metre
        let (out_breach, status_breach) = geofence.enforce(normal_cmd, [0.0, 0.0], 125.0);
        assert!(matches!(status_breach, GeofenceStatus::AltitudeCeilingBreach(_)));
        assert!(out_breach.is_rtl(), "Geofence ihlalinde RTL tetiklenmeli");
    }

    /// Doğrulama 4: MAVLink v2 SET_POSITION_TARGET_LOCAL_NED paket kodlama/çözme
    #[test]
    fn test_mavlink_offboard_stream() {
        let frame = MavlinkV2Frame::create_position_target_local_ned(
            1, 1, 1, 1000, 15.0, -2.0, -0.5, 0.1,
        );

        let bytes = frame.encode(CRC_EXTRA_SET_POSITION_TARGET_LOCAL_NED);
        assert_eq!(bytes[0], mavlink::MAVLINK_V2_STX);

        let decoded = MavlinkV2Frame::decode(&bytes, CRC_EXTRA_SET_POSITION_TARGET_LOCAL_NED)
            .expect("MAVLink decode başarılı olmalı");

        assert_eq!(decoded.msgid, mavlink::MSG_ID_SET_POSITION_TARGET_LOCAL_NED);
        assert_eq!(decoded.seq, 1);

        // Mock araca besle
        let mut mock_vehicle = MockMavlinkVehicle::new();
        mock_vehicle.handle_mavlink_packet(&bytes).expect("Paket işlenmeli");
        assert_eq!(mock_vehicle.velocity_ned, [15.0, -2.0, -0.5]);
    }

    /// Doğrulama 5: VTOL Geçiş Durum Makinesi (Hover -> Transition -> Cruise)
    #[test]
    fn test_vtol_transition_fsm() {
        let mut vtol = VtolTransitionController::new(16.0, 5.0);

        // Başlangıç: Hover
        let mix0 = vtol.update(0.0, 0.1);
        assert_eq!(mix0.state, VtolState::Hover);
        assert_eq!(mix0.mc_throttle_pct, 1.0);
        assert_eq!(mix0.fw_throttle_pct, 0.0);

        // Geçiş başlat
        vtol.request_transition_forward();
        let mix1 = vtol.update(8.0, 2.5);
        assert_eq!(mix1.state, VtolState::TransitionForward);
        assert!(mix1.mc_throttle_pct < 1.0);
        assert!(mix1.fw_throttle_pct > 0.5);

        // Geçiş tamamlandı (süre 5s + hız 18 m/s >= 16 m/s)
        let mix2 = vtol.update(18.0, 3.0);
        assert_eq!(mix2.state, VtolState::CruiseFixedWing);
        assert_eq!(mix2.mc_throttle_pct, 0.0);
        assert_eq!(mix2.fw_throttle_pct, 1.0);
    }

    /// Doğrulama 6: C-ABI 28-Bayt Sıfır Kopyalama
    #[test]
    fn test_bytemuck_c_abi() {
        let setpoint = AutopilotSetpoint::new(10.0, -2.0, 0.5, 0.1, 0.0, AutopilotSetpoint::FLAG_ARMED);
        let bytes = setpoint.as_bytes();
        assert_eq!(bytes.len(), 28);

        let parsed = AutopilotSetpoint::try_from_bytes(bytes).expect("Pod parse");
        assert_eq!(*parsed, setpoint);
    }
}
