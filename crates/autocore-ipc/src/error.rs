use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum IpcError {
    #[error("Shared memory error: {0}")]
    ShmError(String),

    #[error("Buffer pool exhausted: all slots are currently leased")]
    BufferExhausted,

    #[error("Dora node error: {0}")]
    DoraNodeError(String),

    #[error("Fragmentation error: {0}")]
    FragmentationError(String),

    #[error("Reassembly error: {0}")]
    ReassemblyError(String),

    #[error("IPC channel closed or unavailable")]
    ChannelClosed,

    #[error("Payload too large: length {0} exceeds maximum capacity {1}")]
    PayloadTooLarge(usize, usize),

    #[error("CRC-16 checksum mismatch on reassembled payload")]
    ChecksumMismatch,

    #[error("C-ABI zero-copy cast error: {0}")]
    CAbiCastError(String),

    #[error("Incomplete or expired fragment set for message ID {0}")]
    FragmentSetExpired(u32),
}
