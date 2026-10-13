"""
Autocore Python İstemci Paketi
AutoCore Autonomous Robotics Runtime & Safety Kernel
"""

from .structs import (
    SafetyMode,
    KinematicCommand,
    StateVector6DoF,
    ActuatorFeedback,
    BlackboxHeader,
    BlackboxFrame,
)
from .protocol import (
    crc16_ccitt,
    encode_packet,
    decode_packet,
    MSG_TYPE_CMD_VEL,
    MSG_TYPE_FEEDBACK,
    MSG_TYPE_ESTOP,
    MSG_TYPE_HEARTBEAT,
)
from .numpy_bridge import (
    command_to_bytes,
    command_from_bytes,
    command_to_numpy,
    command_from_numpy,
    feedback_to_numpy,
)
from .blackbox import (
    BlackboxReader,
    BLACKBOX_MAGIC,
    BLACKBOX_VERSION,
)
from .client import (
    AutocoreSitlClient,
)

__version__ = "0.1.0"
__all__ = [
    "SafetyMode",
    "KinematicCommand",
    "StateVector6DoF",
    "ActuatorFeedback",
    "BlackboxHeader",
    "BlackboxFrame",
    "crc16_ccitt",
    "encode_packet",
    "decode_packet",
    "MSG_TYPE_CMD_VEL",
    "MSG_TYPE_FEEDBACK",
    "MSG_TYPE_ESTOP",
    "MSG_TYPE_HEARTBEAT",
    "command_to_bytes",
    "command_from_bytes",
    "command_to_numpy",
    "command_from_numpy",
    "feedback_to_numpy",
    "BlackboxReader",
    "BLACKBOX_MAGIC",
    "BLACKBOX_VERSION",
    "AutocoreSitlClient",
]
