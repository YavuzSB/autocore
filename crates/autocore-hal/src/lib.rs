//! # autocore-hal
//!
//! SANCAK Otonom Sistem Mimarisi Donanım Soyutlama Katmanı (HAL).
//! ActuatorPort trait'i, ikili protokol çerçeveleme, CRC-16,
//! seri akış sürücüleri, CAN Bus paketleme ve GPIO emniyet pinleri.

pub mod can;
pub mod gpio;
pub mod protocol;
pub mod serial;
pub mod traits;

pub use can::{decode_can_command, encode_can_command, CanFrame};
pub use gpio::GpioSafetyPin;
pub use protocol::{
    crc16_ccitt, decode_packet, encode_packet, MSG_TYPE_CMD_VEL, MSG_TYPE_ESTOP,
    MSG_TYPE_FEEDBACK, MSG_TYPE_HEARTBEAT,
};
pub use serial::AsyncStreamActuator;
pub use traits::ActuatorPort;
