"""
Autocore Python İkili Protokol Çerçeveleme ve CRC-16 Katmanı
"""

import struct
from typing import Optional, Tuple

SOF_BYTE_1 = 0xAA
SOF_BYTE_2 = 0x55
EOF_BYTE_1 = 0x0D
EOF_BYTE_2 = 0x0A

MSG_TYPE_CMD_VEL = 0x01
MSG_TYPE_FEEDBACK = 0x02
MSG_TYPE_ESTOP = 0x03
MSG_TYPE_HEARTBEAT = 0x04

def crc16_ccitt(data: bytes) -> int:
    """Standart CRC-16-CCITT (Polynom: 0x1021, Başlangıç: 0xFFFF)"""
    crc = 0xFFFF
    for byte in data:
        crc ^= (byte << 8)
        for _ in range(8):
            if (crc & 0x8000) != 0:
                crc = ((crc << 1) ^ 0x1021) & 0xFFFF
            else:
                crc = (crc << 1) & 0xFFFF
    return crc

def encode_packet(msg_type: int, payload: bytes) -> bytes:
    """
    Mesaj tipi ve yükü CRC-16 korumalı pakete sarar:
    [0xAA 0x55][MSG_TYPE(1B)][LEN(2B LE)][PAYLOAD(NB)][CRC(2B LE)][0x0D 0x0A]
    """
    length = len(payload)
    header = bytes([SOF_BYTE_1, SOF_BYTE_2, msg_type]) + struct.pack("<H", length)
    crc_input = bytes([msg_type]) + struct.pack("<H", length) + payload
    crc = crc16_ccitt(crc_input)
    crc_bytes = struct.pack("<H", crc)
    footer = bytes([EOF_BYTE_1, EOF_BYTE_2])
    return header + payload + crc_bytes + footer

def decode_packet(buf: bytearray) -> Optional[Tuple[int, bytes, int]]:
    """
    Gelen bayt akışından geçerli ilk paketi ayrıştırır.
    Dönüş: (msg_type, payload, consumed_bytes) veya None
    """
    if len(buf) < 9:
        return None

    # SOF bul
    sof_idx = -1
    for i in range(len(buf) - 1):
        if buf[i] == SOF_BYTE_1 and buf[i + 1] == SOF_BYTE_2:
            sof_idx = i
            break

    if sof_idx == -1:
        buf.clear()
        return None

    remaining = len(buf) - sof_idx
    if remaining < 9:
        return None

    msg_type = buf[sof_idx + 2]
    payload_len = struct.unpack("<H", buf[sof_idx + 3:sof_idx + 5])[0]
    total_len = 2 + 1 + 2 + payload_len + 2 + 2

    if remaining < total_len:
        return None

    eof_idx = sof_idx + total_len - 2
    if buf[eof_idx] != EOF_BYTE_1 or buf[eof_idx + 1] != EOF_BYTE_2:
        del buf[:sof_idx + 2]
        return None

    crc_pos = eof_idx - 2
    expected_crc = struct.unpack("<H", buf[crc_pos:crc_pos + 2])[0]
    crc_input = bytes(buf[sof_idx + 2:crc_pos])
    computed_crc = crc16_ccitt(crc_input)

    if expected_crc != computed_crc:
        del buf[:sof_idx + 2]
        return None

    payload = bytes(buf[sof_idx + 5:crc_pos])
    consumed = sof_idx + total_len
    return (msg_type, payload, consumed)
