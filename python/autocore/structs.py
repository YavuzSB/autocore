"""
Autocore Python C-ABI Structs & Types
Sancak Autocore Autonomous Robotics Runtime & Safety Kernel

Exact binary layout matching Rust #[repr(C)] structs.
"""

import ctypes
from enum import IntEnum

class SafetyMode(IntEnum):
    BOOTING = 0
    DISARMED = 1
    ARMED = 2
    MISSION = 3
    FAILSAFE = 4
    SAFELOCK = 5
    MANUAL_OVERRIDE = 6

class KinematicCommand(ctypes.Structure):
    """
    Standart Kinematik Hız Komutu (32 Bayt, C-ABI)
    linear: [Vx, Vy, Vz] (m/s)
    angular: [RollRate, PitchRate, YawRate] (rad/s)
    timestamp_us: uint64
    """
    _pack_ = 8
    _fields_ = [
        ("linear", ctypes.c_float * 3),
        ("angular", ctypes.c_float * 3),
        ("timestamp_us", ctypes.c_uint64),
    ]

assert ctypes.sizeof(KinematicCommand) == 32, f"KinematicCommand size must be 32, got {ctypes.sizeof(KinematicCommand)}"

class StateVector6DoF(ctypes.Structure):
    """
    6-DoF Kinematik Durum Kestirim Vektörü (64 Bayt, C-ABI)
    position: [X, Y, Z] (m)
    velocity: [Vx, Vy, Vz] (m/s)
    quaternion: [qw, qx, qy, qz]
    angular_rates: [P, Q, R] (rad/s)
    _pad: 4 bayt dolgu
    timestamp_us: uint64
    """
    _pack_ = 8
    _fields_ = [
        ("position", ctypes.c_float * 3),
        ("velocity", ctypes.c_float * 3),
        ("quaternion", ctypes.c_float * 4),
        ("angular_rates", ctypes.c_float * 3),
        ("_pad", ctypes.c_uint8 * 4),
        ("timestamp_us", ctypes.c_uint64),
    ]

assert ctypes.sizeof(StateVector6DoF) == 64, f"StateVector6DoF size must be 64, got {ctypes.sizeof(StateVector6DoF)}"

class ActuatorFeedback(ctypes.Structure):
    """
    Aktüatör ve Donanım Telemetri Geri Beslemesi (72 Bayt, C-ABI)
    actual_linear: [Vx, Vy, Vz] (m/s)
    actual_angular: [Roll, Pitch, Yaw] (rad/s)
    motor_currents: [M1, M2, M3, M4] (A)
    temperatures: [S1, S2, S3, S4] (°C)
    status_flags: uint32
    _pad: uint32
    timestamp_us: uint64
    """
    _pack_ = 8
    _fields_ = [
        ("actual_linear", ctypes.c_float * 3),
        ("actual_angular", ctypes.c_float * 3),
        ("motor_currents", ctypes.c_float * 4),
        ("temperatures", ctypes.c_float * 4),
        ("status_flags", ctypes.c_uint32),
        ("_pad", ctypes.c_uint32),
        ("timestamp_us", ctypes.c_uint64),
    ]

assert ctypes.sizeof(ActuatorFeedback) == 72, f"ActuatorFeedback size must be 72, got {ctypes.sizeof(ActuatorFeedback)}"

class BlackboxHeader(ctypes.Structure):
    """
    Siyah Kutu Dosya Başlığı (40 Bayt, C-ABI)
    """
    _pack_ = 8
    _fields_ = [
        ("magic", ctypes.c_char * 4),      # b"BBOX"
        ("version", ctypes.c_uint32),      # 1
        ("capacity", ctypes.c_uint32),     # N
        ("record_size", ctypes.c_uint32),  # 184
        ("total_records", ctypes.c_uint64),
        ("start_time_us", ctypes.c_uint64),
        ("end_time_us", ctypes.c_uint64),
    ]

assert ctypes.sizeof(BlackboxHeader) == 40, f"BlackboxHeader size must be 40, got {ctypes.sizeof(BlackboxHeader)}"

class BlackboxFrame(ctypes.Structure):
    """
    Siyah Kutu Telemetri Çerçevesi (184 Bayt, C-ABI)
    """
    _pack_ = 8
    _fields_ = [
        ("timestamp_us", ctypes.c_uint64),
        ("state", StateVector6DoF),
        ("command", KinematicCommand),
        ("feedback", ActuatorFeedback),
        ("crc16", ctypes.c_uint16),
        ("_padding", ctypes.c_uint8 * 6),
    ]

assert ctypes.sizeof(BlackboxFrame) == 184, f"BlackboxFrame size must be 184, got {ctypes.sizeof(BlackboxFrame)}"
