use crate::coordinates::NedPoint;

pub const MAVLINK_V2_STX: u8 = 0xFD;
pub const MSG_ID_HEARTBEAT: u32 = 0;
pub const MSG_ID_LOCAL_POSITION_NED: u32 = 32;
pub const MSG_ID_ATTITUDE_TARGET: u32 = 83;

/// X.25 CRC hesaplaması (MAVLink standardı)
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

/// MAVLink V2 Paket Çerçevesi
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

    /// Çerçeveyi ikili bayt dizisine kodlar.
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

        // 24-bit msgid (3 bayt little-endian)
        buf.push((self.msgid & 0xFF) as u8);
        buf.push(((self.msgid >> 8) & 0xFF) as u8);
        buf.push(((self.msgid >> 16) & 0xFF) as u8);

        buf.extend_from_slice(&self.payload);

        // STX hariç başlık + yük üzerinden CRC
        let crc = mavlink_crc(&buf[1..], crc_extra);
        buf.push((crc & 0xFF) as u8);
        buf.push(((crc >> 8) & 0xFF) as u8);

        buf
    }

    /// Bayt dizisinden MAVLink V2 paketini çözer.
    pub fn decode(bytes: &[u8], crc_extra: u8) -> Result<Self, String> {
        if bytes.len() < 12 {
            return Err("Packet too short for MAVLink v2".into());
        }
        if bytes[0] != MAVLINK_V2_STX {
            return Err(format!("Invalid STX byte: 0x{:02X}", bytes[0]));
        }

        let payload_len = bytes[1] as usize;
        if bytes.len() < 12 + payload_len {
            return Err("Packet length smaller than payload declaration".into());
        }

        let seq = bytes[4];
        let sysid = bytes[5];
        let compid = bytes[6];
        let msgid = (bytes[7] as u32) | ((bytes[8] as u32) << 8) | ((bytes[9] as u32) << 16);

        let payload = bytes[10..10 + payload_len].to_vec();

        let expected_crc = (bytes[10 + payload_len] as u16) | ((bytes[11 + payload_len] as u16) << 8);
        let calc_crc = mavlink_crc(&bytes[1..10 + payload_len], crc_extra);

        if expected_crc != calc_crc {
            return Err("MAVLink CRC mismatch".into());
        }

        Ok(Self {
            seq,
            sysid,
            compid,
            msgid,
            payload,
        })
    }
}

/// MAVLink LOCAL_POSITION_NED Mesajı (ID 32)
#[derive(Debug, Clone, PartialEq)]
pub struct MavlinkLocalPositionNed {
    pub time_boot_ms: u32,
    pub pos: NedPoint,
    pub vel_north: f32,
    pub vel_east: f32,
    pub vel_down: f32,
}

impl MavlinkLocalPositionNed {
    pub const CRC_EXTRA: u8 = 185;

    pub fn to_frame(&self, seq: u8, sysid: u8, compid: u8) -> MavlinkV2Frame {
        let mut payload = Vec::with_capacity(28);
        payload.extend_from_slice(&self.time_boot_ms.to_le_bytes());
        payload.extend_from_slice(&(self.pos.north_m as f32).to_le_bytes());
        payload.extend_from_slice(&(self.pos.east_m as f32).to_le_bytes());
        payload.extend_from_slice(&(self.pos.down_m as f32).to_le_bytes());
        payload.extend_from_slice(&self.vel_north.to_le_bytes());
        payload.extend_from_slice(&self.vel_east.to_le_bytes());
        payload.extend_from_slice(&self.vel_down.to_le_bytes());

        MavlinkV2Frame::new(seq, sysid, compid, MSG_ID_LOCAL_POSITION_NED, payload)
    }

    pub fn from_frame(frame: &MavlinkV2Frame) -> Result<Self, String> {
        if frame.msgid != MSG_ID_LOCAL_POSITION_NED || frame.payload.len() < 28 {
            return Err("Invalid frame for LOCAL_POSITION_NED".into());
        }
        let p = &frame.payload;
        let time_boot_ms = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
        let n = f32::from_le_bytes([p[4], p[5], p[6], p[7]]) as f64;
        let e = f32::from_le_bytes([p[8], p[9], p[10], p[11]]) as f64;
        let d = f32::from_le_bytes([p[12], p[13], p[14], p[15]]) as f64;
        let vn = f32::from_le_bytes([p[16], p[17], p[18], p[19]]);
        let ve = f32::from_le_bytes([p[20], p[21], p[22], p[23]]);
        let vd = f32::from_le_bytes([p[24], p[25], p[26], p[27]]);

        Ok(Self {
            time_boot_ms,
            pos: NedPoint::new(n, e, d),
            vel_north: vn,
            vel_east: ve,
            vel_down: vd,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mavlink_v2_frame_encode_decode() {
        let msg = MavlinkLocalPositionNed {
            time_boot_ms: 12345,
            pos: NedPoint::new(100.5, -50.25, -15.0),
            vel_north: 12.0,
            vel_east: -3.5,
            vel_down: 0.2,
        };

        let frame = msg.to_frame(1, 1, 1);
        let encoded = frame.encode(MavlinkLocalPositionNed::CRC_EXTRA);

        assert_eq!(encoded[0], MAVLINK_V2_STX);

        let decoded_frame = MavlinkV2Frame::decode(&encoded, MavlinkLocalPositionNed::CRC_EXTRA).unwrap();
        assert_eq!(decoded_frame.msgid, MSG_ID_LOCAL_POSITION_NED);

        let decoded_msg = MavlinkLocalPositionNed::from_frame(&decoded_frame).unwrap();
        assert_eq!(decoded_msg.time_boot_ms, 12345);
        assert!((decoded_msg.pos.north_m - 100.5).abs() < 1e-4);
        assert!((decoded_msg.pos.east_m - -50.25).abs() < 1e-4);
    }
}
