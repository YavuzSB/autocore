//! autocore-hal :: CAN Bus ve SocketCAN Çerçeve Yönetimi

use autocore_base::KinematicCommand;

pub const CAN_ID_CMD_VEL_LIN: u32 = 0x100;
pub const CAN_ID_CMD_VEL_ANG: u32 = 0x101;
pub const CAN_ID_ESTOP: u32 = 0x102;
pub const CAN_ID_FEEDBACK_LIN: u32 = 0x200;
pub const CAN_ID_FEEDBACK_MOTORS: u32 = 0x201;

/// Standart / Genişletilmiş CAN Çerçevesi
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanFrame {
    pub id: u32,
    pub is_extended: bool,
    pub dlc: u8,
    pub data: [u8; 8],
}

impl CanFrame {
    pub fn new_standard(id: u32, data: &[u8]) -> Self {
        let mut d = [0u8; 8];
        let len = data.len().min(8);
        d[..len].copy_from_slice(&data[..len]);
        Self {
            id,
            is_extended: false,
            dlc: len as u8,
            data: d,
        }
    }
}

/// Kinematik hız komutunu iki adet 8-baytlık CAN çerçevesine paketler:
/// Frame 1 (0x100): Vx (i16 * 100), Vy (i16 * 100), Vz (i16 * 100)
/// Frame 2 (0x101): RollRate (i16 * 100), PitchRate (i16 * 100), YawRate (i16 * 100)
pub fn encode_can_command(cmd: &KinematicCommand) -> (CanFrame, CanFrame) {
    let mut data_lin = [0u8; 8];
    let vx = (cmd.linear[0] * 100.0) as i16;
    let vy = (cmd.linear[1] * 100.0) as i16;
    let vz = (cmd.linear[2] * 100.0) as i16;
    data_lin[0..2].copy_from_slice(&vx.to_le_bytes());
    data_lin[2..4].copy_from_slice(&vy.to_le_bytes());
    data_lin[4..6].copy_from_slice(&vz.to_le_bytes());
    let frame_lin = CanFrame::new_standard(CAN_ID_CMD_VEL_LIN, &data_lin[..6]);

    let mut data_ang = [0u8; 8];
    let roll = (cmd.angular[0] * 100.0) as i16;
    let pitch = (cmd.angular[1] * 100.0) as i16;
    let yaw = (cmd.angular[2] * 100.0) as i16;
    data_ang[0..2].copy_from_slice(&roll.to_le_bytes());
    data_ang[2..4].copy_from_slice(&pitch.to_le_bytes());
    data_ang[4..6].copy_from_slice(&yaw.to_le_bytes());
    let frame_ang = CanFrame::new_standard(CAN_ID_CMD_VEL_ANG, &data_ang[..6]);

    (frame_lin, frame_ang)
}

/// İki CAN çerçevesinden doğrusal ve açısal hızları geri çözer
pub fn decode_can_command(frame_lin: &CanFrame, frame_ang: &CanFrame) -> Option<KinematicCommand> {
    if frame_lin.id != CAN_ID_CMD_VEL_LIN || frame_ang.id != CAN_ID_CMD_VEL_ANG {
        return None;
    }

    let vx = i16::from_le_bytes([frame_lin.data[0], frame_lin.data[1]]) as f32 / 100.0;
    let vy = i16::from_le_bytes([frame_lin.data[2], frame_lin.data[3]]) as f32 / 100.0;
    let vz = i16::from_le_bytes([frame_lin.data[4], frame_lin.data[5]]) as f32 / 100.0;

    let roll = i16::from_le_bytes([frame_ang.data[0], frame_ang.data[1]]) as f32 / 100.0;
    let pitch = i16::from_le_bytes([frame_ang.data[2], frame_ang.data[3]]) as f32 / 100.0;
    let yaw = i16::from_le_bytes([frame_ang.data[4], frame_ang.data[5]]) as f32 / 100.0;

    Some(KinematicCommand::new([vx, vy, vz], [roll, pitch, yaw], 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_frame_encoding_and_decoding() {
        let cmd = KinematicCommand::new([1.25, -2.50, 0.50], [0.10, -0.20, 1.57], 0);
        let (f_lin, f_ang) = encode_can_command(&cmd);

        assert_eq!(f_lin.id, CAN_ID_CMD_VEL_LIN);
        assert_eq!(f_ang.id, CAN_ID_CMD_VEL_ANG);

        let decoded = decode_can_command(&f_lin, &f_ang).unwrap();
        assert!((decoded.linear[0] - cmd.linear[0]).abs() < 0.02);
        assert!((decoded.linear[1] - cmd.linear[1]).abs() < 0.02);
        assert!((decoded.angular[2] - cmd.angular[2]).abs() < 0.02);
    }
}
