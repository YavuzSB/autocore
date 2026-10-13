use std::collections::HashMap;

use crate::error::IpcError;

pub const DEFAULT_MAX_FRAGMENT_SIZE: usize = 1200;
pub const FRAGMENT_HEADER_SIZE: usize = 12;
pub const FRAGMENT_MAGIC: u16 = 0x5343; // 'S' 'C' (Sync / Shared Core)

/// CRC-16-CCITT hesaplama (Polinom: 0x1021)
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

/// 1200 baytlık MTU sınırına uygun telemetri parçası.
#[derive(Debug, Clone, PartialEq)]
pub struct PacketFragment {
    pub magic: u16,
    pub msg_id: u32,
    pub seq_idx: u16,
    pub total_frags: u16,
    pub crc16: u16,
    pub payload: Vec<u8>,
}

impl PacketFragment {
    pub fn new(msg_id: u32, seq_idx: u16, total_frags: u16, payload: Vec<u8>) -> Self {
        let crc = crc16_ccitt(&payload);
        Self {
            magic: FRAGMENT_MAGIC,
            msg_id,
            seq_idx,
            total_frags,
            crc16: crc,
            payload,
        }
    }

    /// Parçayı ikili (binary) bayt akışına kodlar.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(FRAGMENT_HEADER_SIZE + self.payload.len());
        buf.extend_from_slice(&self.magic.to_be_bytes());
        buf.extend_from_slice(&self.msg_id.to_be_bytes());
        buf.extend_from_slice(&self.seq_idx.to_be_bytes());
        buf.extend_from_slice(&self.total_frags.to_be_bytes());
        buf.extend_from_slice(&self.crc16.to_be_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    /// Bayt akışından parçayı ayrıştırır ve CRC doğrulaması yapar.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, IpcError> {
        if bytes.len() < FRAGMENT_HEADER_SIZE {
            return Err(IpcError::FragmentationError("Buffer too small for fragment header".into()));
        }

        let magic = u16::from_be_bytes([bytes[0], bytes[1]]);
        if magic != FRAGMENT_MAGIC {
            return Err(IpcError::FragmentationError(format!("Invalid magic: 0x{magic:04X}")));
        }

        let msg_id = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);
        let seq_idx = u16::from_be_bytes([bytes[6], bytes[7]]);
        let total_frags = u16::from_be_bytes([bytes[8], bytes[9]]);
        let crc16 = u16::from_be_bytes([bytes[10], bytes[11]]);
        let payload = bytes[FRAGMENT_HEADER_SIZE..].to_vec();

        let calc_crc = crc16_ccitt(&payload);
        if calc_crc != crc16 {
            return Err(IpcError::ChecksumMismatch);
        }

        Ok(Self {
            magic,
            msg_id,
            seq_idx,
            total_frags,
            crc16,
            payload,
        })
    }
}

/// Büyük telemetri paketlerini parçalayıcı (Fragmenter).
pub struct PacketFragmenter {
    max_payload_per_fragment: usize,
}

impl Default for PacketFragmenter {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_FRAGMENT_SIZE)
    }
}

impl PacketFragmenter {
    pub fn new(max_fragment_size: usize) -> Self {
        assert!(max_fragment_size > FRAGMENT_HEADER_SIZE);
        Self {
            max_payload_per_fragment: max_fragment_size - FRAGMENT_HEADER_SIZE,
        }
    }

    pub fn fragment(&self, msg_id: u32, data: &[u8]) -> Vec<PacketFragment> {
        if data.is_empty() {
            return vec![PacketFragment::new(msg_id, 0, 1, Vec::new())];
        }

        let total_frags = ((data.len() + self.max_payload_per_fragment - 1) / self.max_payload_per_fragment) as u16;
        let mut fragments = Vec::with_capacity(total_frags as usize);

        for (idx, chunk) in data.chunks(self.max_payload_per_fragment).enumerate() {
            fragments.push(PacketFragment::new(
                msg_id,
                idx as u16,
                total_frags,
                chunk.to_vec(),
            ));
        }

        fragments
    }
}

/// Parçalanmış telemetri paketlerini yeniden birleştirici (Reassembler Buffer).
pub struct PacketReassembler {
    pending_messages: HashMap<u32, ReassemblyState>,
}

struct ReassemblyState {
    total_frags: u16,
    received_fragments: HashMap<u16, Vec<u8>>,
    created_at_us: u64,
}

impl Default for PacketReassembler {
    fn default() -> Self {
        Self::new()
    }
}

impl PacketReassembler {
    pub fn new() -> Self {
        Self {
            pending_messages: HashMap::new(),
        }
    }

    /// Yeni bir parça ekler. Eğer paket tamamlandıysa tam yükü `Some(Vec<u8>)` olarak döndürür.
    pub fn process_fragment(&mut self, fragment: PacketFragment, now_us: u64) -> Result<Option<Vec<u8>>, IpcError> {
        let msg_id = fragment.msg_id;
        let total_frags = fragment.total_frags;

        if fragment.seq_idx >= total_frags {
            return Err(IpcError::ReassemblyError(format!(
                "Sequence index {} exceeds total fragments {}",
                fragment.seq_idx, total_frags
            )));
        }

        let state = self.pending_messages.entry(msg_id).or_insert_with(|| ReassemblyState {
            total_frags,
            received_fragments: HashMap::with_capacity(total_frags as usize),
            created_at_us: now_us,
        });

        if state.total_frags != total_frags {
            return Err(IpcError::ReassemblyError(
                "Mismatched total_frags for same msg_id".into(),
            ));
        }

        state.received_fragments.insert(fragment.seq_idx, fragment.payload);

        if state.received_fragments.len() == state.total_frags as usize {
            // Tüm parçalar geldi, sırayla birleştir
            let state = self.pending_messages.remove(&msg_id).unwrap();
            let mut full_payload = Vec::new();
            for seq in 0..state.total_frags {
                if let Some(chunk) = state.received_fragments.get(&seq) {
                    full_payload.extend_from_slice(chunk);
                } else {
                    return Err(IpcError::ReassemblyError(format!("Missing sequence index {seq}")));
                }
            }
            Ok(Some(full_payload))
        } else {
            Ok(None)
        }
    }

    /// Zaman aşımına uğramış eksik mesajları tampondan temizler.
    pub fn cleanup_expired(&mut self, now_us: u64, timeout_us: u64) -> usize {
        let initial_len = self.pending_messages.len();
        self.pending_messages
            .retain(|_, state| now_us.saturating_sub(state.created_at_us) <= timeout_us);
        initial_len - self.pending_messages.len()
    }

    pub fn pending_count(&self) -> usize {
        self.pending_messages.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fragment_binary_roundtrip() {
        let payload = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let frag = PacketFragment::new(1001, 0, 1, payload.clone());
        let bytes = frag.to_bytes();
        let decoded = PacketFragment::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.msg_id, 1001);
        assert_eq!(decoded.seq_idx, 0);
        assert_eq!(decoded.total_frags, 1);
        assert_eq!(decoded.payload, payload);
    }

    #[test]
    fn test_packet_fragmentation_and_reassembly() {
        let fragmenter = PacketFragmenter::new(50); // Küçük MTU: 50 bayt
        let reassembler = &mut PacketReassembler::new();

        // 120 baytlık veri
        let original_data: Vec<u8> = (0..120).map(|i| (i % 256) as u8).collect();
        let fragments = fragmenter.fragment(42, &original_data);

        assert!(fragments.len() > 1);

        let mut completed = None;
        for frag in fragments {
            if let Some(payload) = reassembler.process_fragment(frag, 1000).unwrap() {
                completed = Some(payload);
            }
        }

        assert_eq!(completed, Some(original_data));
        assert_eq!(reassembler.pending_count(), 0);
    }

    #[test]
    fn test_reassembler_cleanup_expired() {
        let mut reassembler = PacketReassembler::new();
        let frag = PacketFragment::new(99, 0, 2, vec![1, 2, 3]);
        let res = reassembler.process_fragment(frag, 1000).unwrap();
        assert_eq!(res, None);
        assert_eq!(reassembler.pending_count(), 1);

        // 500 us timeout: 1600 anında (fark 600) temizlenmeli
        let dropped = reassembler.cleanup_expired(1600, 500);
        assert_eq!(dropped, 1);
        assert_eq!(reassembler.pending_count(), 0);
    }
}
