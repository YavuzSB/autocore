"""
Autocore Python Siyah Kutu (Blackbox) İkili Kayıt Ayrıştırıcısı
Rust/C++ tarafından üretilen 184 baytlık telemetri bloklarını okur ve doğrular.
"""

import ctypes
from typing import List, Tuple
from .structs import BlackboxHeader, BlackboxFrame
from .protocol import crc16_ccitt

BLACKBOX_MAGIC = b"BBOX"
BLACKBOX_VERSION = 1

class BlackboxReader:
    @staticmethod
    def load_from_file(file_path: str) -> Tuple[BlackboxHeader, List[BlackboxFrame]]:
        with open(file_path, "rb") as f:
            data = f.read()

        header_size = ctypes.sizeof(BlackboxHeader)
        if len(data) < header_size:
            raise ValueError(f"Geçersiz dosya uzunluğu: en az {header_size} bayt olmalı")

        header = BlackboxHeader.from_buffer_copy(data[:header_size])
        if header.magic != BLACKBOX_MAGIC:
            raise ValueError(f"Geçersiz sihirli baytlar (magic bytes): {header.magic}")

        if header.version != BLACKBOX_VERSION:
            raise ValueError(f"Desteklenmeyen blackbox sürümü: {header.version}")

        frame_size = ctypes.sizeof(BlackboxFrame)
        frame_bytes = data[header_size:]

        if len(frame_bytes) % frame_size != 0:
            raise ValueError(f"Çerçeve hizalama hatası: kalan veri {len(frame_bytes)} bayt {frame_size}'in katı değil")

        num_frames = len(frame_bytes) // frame_size
        frames = []

        for i in range(num_frames):
            chunk = frame_bytes[i * frame_size : (i + 1) * frame_size]
            frame = BlackboxFrame.from_buffer_copy(chunk)

            # İlk 176 bayt üzerinden CRC doğrulaması
            computed_crc = crc16_ccitt(chunk[:176])
            if frame.crc16 != computed_crc:
                raise ValueError(
                    f"Çerçeve {i} CRC hatası! Beklenen {computed_crc:#06x}, okunan {frame.crc16:#06x}"
                )
            frames.append(frame)

        return header, frames

    @staticmethod
    def to_dict_list(frames: List[BlackboxFrame]) -> List[dict]:
        """Çerçeveleri analiz ve çizim için sözlük listesine dönüştürür"""
        rows = []
        for f in frames:
            rows.append({
                "timestamp_us": f.timestamp_us,
                "pos_x": f.state.position[0],
                "pos_y": f.state.position[1],
                "pos_z": f.state.position[2],
                "vel_x": f.state.velocity[0],
                "vel_y": f.state.velocity[1],
                "vel_z": f.state.velocity[2],
                "cmd_vx": f.command.linear[0],
                "cmd_vy": f.command.linear[1],
                "cmd_vz": f.command.linear[2],
                "cmd_yaw_rate": f.command.angular[2],
                "fb_curr0": f.feedback.motor_currents[0],
                "fb_curr1": f.feedback.motor_currents[1],
                "fb_temp0": f.feedback.temperatures[0],
                "fb_temp1": f.feedback.temperatures[1],
                "status_flags": f.feedback.status_flags,
            })
        return rows
