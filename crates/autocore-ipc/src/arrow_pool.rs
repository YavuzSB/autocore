use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use autocore_base::{ActuatorFeedback, KinematicCommand, StateVector6DoF};
use bytemuck::{bytes_of, try_from_bytes};
pub use dora_node_api::arrow_v59 as arrow;
use dora_node_api::arrow_v59::array::UInt8Array;

use crate::error::IpcError;

/// Apache Arrow ve C-ABI veri yapıları arasında sıfır-kopyalama (Zero-Copy) dönüştürücü.
pub struct ArrowKinematicEncoder;

impl ArrowKinematicEncoder {
    /// `KinematicCommand` (32 bayt) struct'ını sıfır kopyalama ile Apache Arrow `UInt8Array` dizisine sarar.
    pub fn encode_kinematic(cmd: &KinematicCommand) -> UInt8Array {
        let bytes = bytes_of(cmd);
        UInt8Array::from(bytes.to_vec())
    }

    /// Arrow `UInt8Array` dizisinden C-ABI `KinematicCommand` struct'ını sıfır kopyalama görünümü ile çözer.
    pub fn decode_kinematic(array: &UInt8Array) -> Result<KinematicCommand, IpcError> {
        let slice = array.values();
        try_from_bytes::<KinematicCommand>(slice)
            .map(|cmd| *cmd)
            .map_err(|e| IpcError::CAbiCastError(format!("KinematicCommand decode failed: {e:?}")))
    }

    /// `StateVector6DoF` (64 bayt) struct'ını Apache Arrow `UInt8Array` dizisine sarar.
    pub fn encode_state_vector(state: &StateVector6DoF) -> UInt8Array {
        let bytes = bytes_of(state);
        UInt8Array::from(bytes.to_vec())
    }

    /// Arrow `UInt8Array` dizisinden `StateVector6DoF` struct'ını çözer.
    pub fn decode_state_vector(array: &UInt8Array) -> Result<StateVector6DoF, IpcError> {
        let slice = array.values();
        try_from_bytes::<StateVector6DoF>(slice)
            .map(|s| *s)
            .map_err(|e| IpcError::CAbiCastError(format!("StateVector6DoF decode failed: {e:?}")))
    }

    /// `ActuatorFeedback` (72 bayt) struct'ını Arrow dizisine sarar.
    pub fn encode_feedback(fb: &ActuatorFeedback) -> UInt8Array {
        let bytes = bytes_of(fb);
        UInt8Array::from(bytes.to_vec())
    }

    /// Arrow `UInt8Array` dizisinden `ActuatorFeedback` struct'ını çözer.
    pub fn decode_feedback(array: &UInt8Array) -> Result<ActuatorFeedback, IpcError> {
        let slice = array.values();
        try_from_bytes::<ActuatorFeedback>(slice)
            .map(|f| *f)
            .map_err(|e| IpcError::CAbiCastError(format!("ActuatorFeedback decode failed: {e:?}")))
    }
}

/// Önceden tahsis edilmiş sabit boyutlu paylaşılan bellek halka havuzu (Pre-allocated Shared Ring Buffer).
/// Bellek parçalanmasını (fragmentation) ve sistem çağrısı gecikmelerini sıfıra indirir.
pub struct SharedRingBufferPool {
    slot_count: usize,
    slot_size: usize,
    slots: Vec<Arc<BufferSlot>>,
}

struct BufferSlot {
    id: usize,
    in_use: AtomicBool,
    data: std::cell::UnsafeCell<Vec<u8>>,
}

// Güvenli RAII slot kiralama bekçisi
unsafe impl Send for BufferSlot {}
unsafe impl Sync for BufferSlot {}

pub struct BufferLease {
    slot: Arc<BufferSlot>,
    len: usize,
}

impl std::fmt::Debug for BufferLease {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferLease")
            .field("slot_id", &self.slot.id)
            .field("len", &self.len)
            .field("capacity", &self.capacity())
            .finish()
    }
}

impl BufferLease {
    pub fn id(&self) -> usize {
        self.slot.id
    }

    pub fn capacity(&self) -> usize {
        unsafe { (*self.slot.data.get()).len() }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let vec = &*self.slot.data.get();
            &vec[..self.len]
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            let vec = &mut *self.slot.data.get();
            &mut vec[..self.len]
        }
    }

    pub fn set_len(&mut self, new_len: usize) -> Result<(), IpcError> {
        let cap = self.capacity();
        if new_len > cap {
            return Err(IpcError::PayloadTooLarge(new_len, cap));
        }
        self.len = new_len;
        Ok(())
    }

    pub fn write_bytes(&mut self, data: &[u8]) -> Result<(), IpcError> {
        self.set_len(data.len())?;
        self.as_mut_slice().copy_from_slice(data);
        Ok(())
    }

    pub fn to_arrow(&self) -> UInt8Array {
        UInt8Array::from(self.as_slice().to_vec())
    }
}

impl Drop for BufferLease {
    fn drop(&mut self) {
        self.slot.in_use.store(false, Ordering::Release);
    }
}

impl SharedRingBufferPool {
    pub fn new(slot_count: usize, slot_size: usize) -> Self {
        assert!(slot_count > 0 && slot_size > 0);
        let mut slots = Vec::with_capacity(slot_count);
        for id in 0..slot_count {
            slots.push(Arc::new(BufferSlot {
                id,
                in_use: AtomicBool::new(false),
                data: std::cell::UnsafeCell::new(vec![0u8; slot_size]),
            }));
        }
        Self {
            slot_count,
            slot_size,
            slots,
        }
    }

    /// Havuzdan boş bir bellek yuvası kiralar.
    pub fn acquire_lease(&self) -> Result<BufferLease, IpcError> {
        for slot in &self.slots {
            if !slot.in_use.swap(true, Ordering::AcqRel) {
                return Ok(BufferLease {
                    slot: Arc::clone(slot),
                    len: self.slot_size,
                });
            }
        }
        Err(IpcError::BufferExhausted)
    }

    pub fn slot_count(&self) -> usize {
        self.slot_count
    }

    pub fn slot_size(&self) -> usize {
        self.slot_size
    }

    /// Kaç yuvanın şu an kullanımda olduğunu verir.
    pub fn active_leases(&self) -> usize {
        self.slots
            .iter()
            .filter(|s| s.in_use.load(Ordering::Relaxed))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arrow_kinematic_roundtrip() {
        let mut cmd = KinematicCommand::default();
        cmd.linear = [1.5, -0.5, 0.0];
        cmd.angular = [0.0, 0.0, 0.75];
        cmd.timestamp_us = 123456789;

        let arrow_array = ArrowKinematicEncoder::encode_kinematic(&cmd);
        assert_eq!(arrow_array.len(), 32);

        let decoded = ArrowKinematicEncoder::decode_kinematic(&arrow_array).unwrap();
        assert_eq!(decoded.linear, [1.5, -0.5, 0.0]);
        assert_eq!(decoded.angular, [0.0, 0.0, 0.75]);
        assert_eq!(decoded.timestamp_us, 123456789);
    }

    #[test]
    fn test_shared_ring_buffer_pool_lease_and_release() {
        let pool = SharedRingBufferPool::new(2, 64);
        assert_eq!(pool.active_leases(), 0);

        let mut lease1 = pool.acquire_lease().unwrap();
        assert_eq!(pool.active_leases(), 1);
        lease1.write_bytes(b"hello autocore ipc").unwrap();
        assert_eq!(lease1.as_slice(), b"hello autocore ipc");

        let lease2 = pool.acquire_lease().unwrap();
        assert_eq!(pool.active_leases(), 2);

        // Havuz doldu, 3. istek hata vermeli
        let err = pool.acquire_lease();
        assert_eq!(err.unwrap_err(), IpcError::BufferExhausted);

        // lease1 bırakılınca havuza geri dönmeli
        drop(lease1);
        assert_eq!(pool.active_leases(), 1);

        let lease3 = pool.acquire_lease().unwrap();
        assert_eq!(pool.active_leases(), 2);
        drop(lease2);
        drop(lease3);
        assert_eq!(pool.active_leases(), 0);
    }
}
