"""
Autocore Python SITL & Seri İstemci Kiti (AutocoreSitlClient)
Linux POSIX dosya tanıtıcısı üzerinden sıfır bağımlılıkla PTY/Seri port ile haberleşir.
"""

import os
import time
from typing import Optional
from .structs import KinematicCommand, ActuatorFeedback, SafetyMode
from .protocol import (
    encode_packet,
    decode_packet,
    MSG_TYPE_CMD_VEL,
    MSG_TYPE_ESTOP,
    MSG_TYPE_FEEDBACK,
)
from .numpy_bridge import command_to_bytes

class AutocoreSitlClient:
    def __init__(self, port_path: str):
        self.port_path = port_path
        # Linux raw blocking/non-blocking POSIX open
        self.fd = os.open(port_path, os.O_RDWR | os.O_NOCTTY)
        self.read_buffer = bytearray()
        self.latest_feedback: Optional[ActuatorFeedback] = None
        self.safety_mode: SafetyMode = SafetyMode.DISARMED

    def send_command(self, cmd: KinematicCommand) -> None:
        """Kinematik hız komutunu paketleyip sanal porta basar"""
        payload = command_to_bytes(cmd)
        packet = encode_packet(MSG_TYPE_CMD_VEL, payload)
        os.write(self.fd, packet)

    def send_cmd_vel(
        self,
        vx: float = 0.0,
        vy: float = 0.0,
        vz: float = 0.0,
        roll: float = 0.0,
        pitch: float = 0.0,
        yaw: float = 0.0,
        timestamp_us: int = 0,
    ) -> None:
        """Kolay hız komutu gönderimi"""
        if timestamp_us == 0:
            timestamp_us = int(time.time() * 1_000_000)
        cmd = KinematicCommand()
        cmd.linear[0] = float(vx)
        cmd.linear[1] = float(vy)
        cmd.linear[2] = float(vz)
        cmd.angular[0] = float(roll)
        cmd.angular[1] = float(pitch)
        cmd.angular[2] = float(yaw)
        cmd.timestamp_us = timestamp_us
        self.send_command(cmd)

    def emergency_stop(self) -> None:
        """Donanıma acil durdurma (E-Stop) sinyali gönderir"""
        self.safety_mode = SafetyMode.SAFELOCK
        packet = encode_packet(MSG_TYPE_ESTOP, b"\xFF")
        os.write(self.fd, packet)

    def poll_feedback(self, timeout_sec: float = 0.05) -> Optional[ActuatorFeedback]:
        """Portu dinleyip en son gelen ActuatorFeedback telemetrisini ayrıştırır"""
        start = time.time()
        while time.time() - start < timeout_sec:
            try:
                # Bloklamadan veya kısa bekleme ile oku
                chunk = os.read(self.fd, 256)
                if chunk:
                    self.read_buffer.extend(chunk)
            except BlockingIOError:
                pass

            # Tamponu çöz
            parsed = decode_packet(self.read_buffer)
            if parsed is not None:
                msg_type, payload, consumed = parsed
                del self.read_buffer[:consumed]
                if msg_type == MSG_TYPE_FEEDBACK and len(payload) == 72:
                    self.latest_feedback = ActuatorFeedback.from_buffer_copy(payload)
                    return self.latest_feedback

            time.sleep(0.005)

        return self.latest_feedback

    def close(self) -> None:
        if self.fd is not None:
            os.close(self.fd)
            self.fd = None

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.close()
