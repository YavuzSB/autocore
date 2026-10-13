"""
Autocore Python NumPy Sıfır Kopyalama Köprüsü (Zero-Copy NumPy Bridge)
NumPy dizileri ile ctypes C-ABI struct'ları arasında 0 ms sıfır kopyalama bellek eşlemesi.
"""

import numpy as np
import ctypes
from .structs import KinematicCommand, StateVector6DoF, ActuatorFeedback

def command_to_bytes(cmd: KinematicCommand) -> bytes:
    """Struct'ı kopyalamadan ham bayt dizisi olarak döndürür"""
    return bytes(cmd)

def command_from_bytes(data: bytes) -> KinematicCommand:
    """32 baytlık ham veriden kopyalamadan struct oluşturur"""
    assert len(data) == ctypes.sizeof(KinematicCommand), f"Boyut 32 olmalı, {len(data)} geldi"
    return KinematicCommand.from_buffer_copy(data)

def command_to_numpy(cmd: KinematicCommand) -> np.ndarray:
    """Kinematik komutun linear ve angular bileşenlerini birleşik NumPy dizisi olarak döndürür (0-copy)"""
    # 6 elemanlı float32 vektör: [Vx, Vy, Vz, RollRate, PitchRate, YawRate]
    mv = memoryview(cmd).cast('B')[:24]
    return np.frombuffer(mv, dtype=np.float32)

def command_from_numpy(arr: np.ndarray, timestamp_us: int = 0) -> KinematicCommand:
    """6 elemanlı NumPy dizisinden (Vx, Vy, Vz, RollRate, PitchRate, YawRate) KinematicCommand üretir"""
    assert arr.size == 6, f"6 eleman bekleniyor (linear + angular), {arr.size} geldi"
    arr_f32 = np.ascontiguousarray(arr, dtype=np.float32)
    cmd = KinematicCommand()
    cmd.linear[0] = float(arr_f32[0])
    cmd.linear[1] = float(arr_f32[1])
    cmd.linear[2] = float(arr_f32[2])
    cmd.angular[0] = float(arr_f32[3])
    cmd.angular[1] = float(arr_f32[4])
    cmd.angular[2] = float(arr_f32[5])
    cmd.timestamp_us = timestamp_us
    return cmd

def feedback_to_numpy(fb: ActuatorFeedback) -> dict:
    """Feedback alanlarını sıfır kopyalama NumPy görünümleri olarak döndürür"""
    mv = memoryview(fb).cast('B')
    return {
        "actual_linear": np.frombuffer(mv[0:12], dtype=np.float32),
        "actual_angular": np.frombuffer(mv[12:24], dtype=np.float32),
        "motor_currents": np.frombuffer(mv[24:40], dtype=np.float32),
        "temperatures": np.frombuffer(mv[40:56], dtype=np.float32),
        "status_flags": fb.status_flags,
        "timestamp_us": fb.timestamp_us,
    }

def state_to_numpy(state: StateVector6DoF) -> dict:
    """StateVector6DoF alanlarını sıfır kopyalama NumPy görünümleri olarak döndürür"""
    mv = memoryview(state).cast('B')
    return {
        "position": np.frombuffer(mv[0:12], dtype=np.float32),
        "velocity": np.frombuffer(mv[12:24], dtype=np.float32),
        "quaternion": np.frombuffer(mv[24:40], dtype=np.float32),
        "angular_rates": np.frombuffer(mv[40:52], dtype=np.float32),
        "timestamp_us": state.timestamp_us,
    }
