//! autocore-hal :: Seri/CAN İkili Protokol Çerçeveleme ve CRC-16 Katmanı

use autocore_base::HalError;

pub const SOF_BYTE_1: u8 = 0xAA;
pub const SOF_BYTE_2: u8 = 0x55;
pub const EOF_BYTE_1: u8 = 0x0D;
pub const EOF_BYTE_2: u8 = 0x0A;

pub const MSG_TYPE_CMD_VEL: u8 = 0x01;
pub const MSG_TYPE_FEEDBACK: u8 = 0x02;
pub const MSG_TYPE_ESTOP: u8 = 0x03;
pub const MSG_TYPE_HEARTBEAT: u8 = 0x04;

/// Standart CRC-16-CCITT (Polynom: 0x1021, Başlangıç: 0xFFFF)
pub fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Verilen mesaj türü ve yükü CRC-16 korumalı pakete sarar:
/// [SOF(2B)][MSG_TYPE(1B)][LEN(2B LE)][PAYLOAD(N B)][CRC(2B LE)][EOF(2B)]
pub fn encode_packet(msg_type: u8, payload: &[u8]) -> Vec<u8> {
    let len = payload.len() as u16;
    let mut packet = Vec::with_capacity(2 + 1 + 2 + payload.len() + 2 + 2);

    packet.push(SOF_BYTE_1);
    packet.push(SOF_BYTE_2);
    packet.push(msg_type);
    packet.extend_from_slice(&len.to_le_bytes());
    packet.extend_from_slice(payload);

    // CRC hesaplaması: MSG_TYPE + LEN + PAYLOAD
    let crc_data_start = 2;
    let crc = crc16_ccitt(&packet[crc_data_start..]);
    packet.extend_from_slice(&crc.to_le_bytes());

    packet.push(EOF_BYTE_1);
    packet.push(EOF_BYTE_2);

    packet
}

/// Gelen bayt tamponundan geçerli bir paket ayrıştırır.
/// Dönüş: `Ok(Some((msg_type, payload, consumed_bytes)))` veya `Ok(None)` (paket henüz tamamlanmadıysa)
pub fn decode_packet(buf: &[u8]) -> Result<Option<(u8, Vec<u8>, usize)>, HalError> {
    if buf.len() < 9 {
        // En küçük paket: SOF(2) + TYPE(1) + LEN(2) + CRC(2) + EOF(2) = 9 bayt
        return Ok(None);
    }

    // SOF ara
    let mut start_idx = 0;
    while start_idx + 1 < buf.len() {
        if buf[start_idx] == SOF_BYTE_1 && buf[start_idx + 1] == SOF_BYTE_2 {
            break;
        }
        start_idx += 1;
    }

    if start_idx + 1 >= buf.len() {
        return Ok(None);
    }

    let remaining = &buf[start_idx..];
    if remaining.len() < 9 {
        return Ok(None);
    }

    let msg_type = remaining[2];
    let payload_len = u16::from_le_bytes([remaining[3], remaining[4]]) as usize;
    let total_packet_len = 2 + 1 + 2 + payload_len + 2 + 2;

    if remaining.len() < total_packet_len {
        // Paketin tamamı henüz gelmedi
        return Ok(None);
    }

    // EOF kontrolü
    let eof_pos = total_packet_len - 2;
    if remaining[eof_pos] != EOF_BYTE_1 || remaining[eof_pos + 1] != EOF_BYTE_2 {
        // Bozuk çerçeve, 1 bayt ilerletip tekrar dene
        return Err(HalError::SerialError(
            "Geçersiz paket sonlandırıcı (EOF)".into(),
        ));
    }

    // CRC doğrulaması
    let payload = remaining[5..5 + payload_len].to_vec();
    let expected_crc = u16::from_le_bytes([remaining[5 + payload_len], remaining[5 + payload_len + 1]]);
    let calculated_crc = crc16_ccitt(&remaining[2..5 + payload_len]);

    if expected_crc != calculated_crc {
        return Err(HalError::ChecksumMismatch);
    }

    let consumed = start_idx + total_packet_len;
    Ok(Some((msg_type, payload, consumed)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_encode_decode_roundtrip() {
        let payload = b"SANCAK_ROBOTIK_TEST_PAYLOAD";
        let encoded = encode_packet(MSG_TYPE_CMD_VEL, payload);

        let decoded = decode_packet(&encoded).unwrap();
        assert!(decoded.is_some());
        let (msg_type, dec_payload, consumed) = decoded.unwrap();

        assert_eq!(msg_type, MSG_TYPE_CMD_VEL);
        assert_eq!(&dec_payload, payload);
        assert_eq!(consumed, encoded.len());
    }

    #[test]
    fn test_corrupted_crc() {
        let payload = b"TEST";
        let mut encoded = encode_packet(MSG_TYPE_CMD_VEL, payload);

        // Bir baytı boz
        encoded[5] ^= 0xFF;

        let res = decode_packet(&encoded);
        assert_eq!(res, Err(HalError::ChecksumMismatch));
    }
}
