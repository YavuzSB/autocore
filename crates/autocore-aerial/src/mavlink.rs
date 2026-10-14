//! autocore-aerial :: MAVLink v2 Otopilot İletişim Köprüsü (MavlinkBridge)
//!
//! PX4 ve ArduPilot otopilotları ile SET_POSITION_TARGET_LOCAL_NED,
//! HEARTBEAT ve RTL komutlarını ikili bayt dizisine kodlar ve çözer.

pub const MAVLINK_V2_STX: u8 = 0xFD;

pub const MSG_ID_HEARTBEAT: u32 = 0;
pub const MSG_ID_COMMAND_LONG: u32 = 76;
pub const MSG_ID_SET_POSITION_TARGET_LOCAL_NED: u32 = 84;

pub const CRC_EXTRA_HEARTBEAT: u8 = 50;
pub const CRC_EXTRA_COMMAND_LONG: u8 = 152;
pub const CRC_EXTRA_SET_POSITION_TARGET_LOCAL_NED: u8 = 143;

pub const MAV_CMD_NAV_RETURN_TO_LAUNCH: u16 = 20;

/// MAVLink X.25 CRC Birikimli Hesaplayıcısı
pub fn mavlink_crc_accumulate(byte: u8, mut crc: u16) -> u16 {
    let mut tmp = (byte as u16) ^ (crc & 0xFF);
    tmp ^= (tmp << 4) & 0xFF;
    crc = (crc >> 8) ^ (tmp << 8) ^ (tmp << 3) ^ (tmp >> 4);
    crc
}

pub fn mavlink_crc(data: &[u8], crc_extra: u8) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc = mavlink_crc_accumulate(b, crc);
    }
    mavlink_crc_accumulate(crc_extra, crc)
}

/// MAVLink v2 Paket Çerçevesi
#[derive(Debug, Clone, PartialEq)]
pub struct MavlinkV2Frame {
    pub seq: u8,
    pub sysid: u8,
    pub compid: u8,
    pub msgid: u32,
    pub payload: Vec<u8>,
}

impl MavlinkV2Frame {
    pub fn new(seq: u8, sysid: u8, compid: u8, msgid: u32, payload: Vec<u8>) -> Self {
        Self {
            seq,
            sysid,
            compid,
            msgid,
            payload,
        }
    }

    /// Çerçeveyi ikili MAVLink v2 bayt dizisine kodlar
    pub fn encode(&self, crc_extra: u8) -> Vec<u8> {
        let payload_len = self.payload.len() as u8;
        let mut buf = Vec::with_capacity(12 + self.payload.len());

        buf.push(MAVLINK_V2_STX);
        buf.push(payload_len);
        buf.push(0); // incompat_flags
        buf.push(0); // compat_flags
        buf.push(self.seq);
        buf.push(self.sysid);
        buf.push(self.compid);

        // 24-bit MsgID (Little Endian)
        buf.push((self.msgid & 0xFF) as u8);
        buf.push(((self.msgid >> 8) & 0xFF) as u8);
        buf.push(((self.msgid >> 16) & 0xFF) as u8);

        // Payload
        buf.extend_from_slice(&self.payload);

        // CRC hesabı (STX hariç 1..10 + payload)
        let crc = mavlink_crc(&buf[1..], crc_extra);
        buf.push((crc & 0xFF) as u8);
        buf.push(((crc >> 8) & 0xFF) as u8);

        buf
    }

    /// Ham bayt dizisinden MAVLink v2 paketini çözer (parse)
    pub fn decode(bytes: &[u8], crc_extra: u8) -> Result<Self, &'static str> {
        if bytes.len() < 12 {
            return Err("Paket çok kısa");
        }
        if bytes[0] != MAVLINK_V2_STX {
            return Err("Geçersiz MAVLink v2 STX");
        }

        let payload_len = bytes[1] as usize;
        let expected_total_len = 10 + payload_len + 2;
        if bytes.len() < expected_total_len {
            return Err("Eksik bayt uzunluğu");
        }

        let seq = bytes[4];
        let sysid = bytes[5];
        let compid = bytes[6];
        let msgid = (bytes[7] as u32) | ((bytes[8] as u32) << 8) | ((bytes[9] as u32) << 16);

        let payload = bytes[10..10 + payload_len].to_vec();

        // CRC kontrolü
        let received_crc = (bytes[10 + payload_len] as u16)
            | ((bytes[10 + payload_len + 1] as u16) << 8);
        let computed_crc = mavlink_crc(&bytes[1..10 + payload_len], crc_extra);

        if received_crc != computed_crc {
            return Err("CRC eşleşmedi");
        }

        Ok(Self {
            seq,
            sysid,
            compid,
            msgid,
            payload,
        })
    }

    /// SET_POSITION_TARGET_LOCAL_NED paketinden [vx, vy, vz] hızlarını okur
    pub fn extract_velocity_ned(&self) -> Option<[f32; 3]> {
        if self.msgid == MSG_ID_SET_POSITION_TARGET_LOCAL_NED && self.payload.len() >= 31 {
            let vx = f32::from_le_bytes(self.payload[19..23].try_into().ok()?);
            let vy = f32::from_le_bytes(self.payload[23..27].try_into().ok()?);
            let vz = f32::from_le_bytes(self.payload[27..31].try_into().ok()?);
            Some([vx, vy, vz])
        } else {
            None
        }
    }

    /// SET_POSITION_TARGET_LOCAL_NED (MsgID 84) paketi üretir
    pub fn create_position_target_local_ned(
        seq: u8,
        sysid: u8,
        compid: u8,
        time_boot_ms: u32,
        vx: f32,
        vy: f32,
        vz: f32,
        yaw_rate: f32,
    ) -> Self {
        let mut payload = Vec::with_capacity(53);
        payload.extend_from_slice(&time_boot_ms.to_le_bytes());
        // coordinate_frame: 1 = MAV_FRAME_LOCAL_NED
        payload.push(1);
        // type_mask: hız ve yaw_rate aktif (bitmask 0b0000_0101_1100_0111 = 0x05C7)
        let type_mask: u16 = 0x05C7;
        payload.extend_from_slice(&type_mask.to_le_bytes());

        // x, y, z (konum pasif, 0.0)
        payload.extend_from_slice(&0.0_f32.to_le_bytes());
        payload.extend_from_slice(&0.0_f32.to_le_bytes());
        payload.extend_from_slice(&0.0_f32.to_le_bytes());

        // vx, vy, vz
        payload.extend_from_slice(&vx.to_le_bytes());
        payload.extend_from_slice(&vy.to_le_bytes());
        payload.extend_from_slice(&vz.to_le_bytes());

        // afx, afy, afz
        payload.extend_from_slice(&0.0_f32.to_le_bytes());
        payload.extend_from_slice(&0.0_f32.to_le_bytes());
        payload.extend_from_slice(&0.0_f32.to_le_bytes());

        // yaw, yaw_rate
        payload.extend_from_slice(&0.0_f32.to_le_bytes());
        payload.extend_from_slice(&yaw_rate.to_le_bytes());

        Self::new(seq, sysid, compid, MSG_ID_SET_POSITION_TARGET_LOCAL_NED, payload)
    }

    /// MAV_CMD_NAV_RETURN_TO_LAUNCH RTL komut paketi üretir
    pub fn create_rtl_command(seq: u8, sysid: u8, compid: u8) -> Self {
        let mut payload = Vec::with_capacity(33);
        // param 1..7 (f32)
        for _ in 0..7 {
            payload.extend_from_slice(&0.0_f32.to_le_bytes());
        }
        // command: MAV_CMD_NAV_RETURN_TO_LAUNCH = 20 (u16)
        payload.extend_from_slice(&MAV_CMD_NAV_RETURN_TO_LAUNCH.to_le_bytes());
        // target_system, target_component, confirmation
        payload.push(1);
        payload.push(1);
        payload.push(0);

        Self::new(seq, sysid, compid, MSG_ID_COMMAND_LONG, payload)
    }
}
