pub mod arrow_pool;
pub mod drop_old;
pub mod error;
pub mod node;
pub mod packet;

pub use arrow_pool::{ArrowKinematicEncoder, BufferLease, SharedRingBufferPool};
pub use drop_old::{DropOldLeaseQueue, DropOldMailbox};
pub use error::IpcError;
pub use node::{AutocoreEvent, DoraNodeWrapper};
pub use packet::{PacketFragment, PacketFragmenter, PacketReassembler};

#[cfg(test)]
mod tests {
    use super::*;
    use autocore_base::KinematicCommand;

    #[tokio::test]
    async fn test_ipc_end_to_end_pipeline() {
        // 1. Arrow Zero-Copy pool
        let pool = SharedRingBufferPool::new(4, 1024);
        let mut lease = pool.acquire_lease().unwrap();

        let mut cmd = KinematicCommand::default();
        cmd.linear = [1.0, 2.0, 3.0];
        cmd.angular = [0.1, 0.2, 0.3];
        cmd.timestamp_us = 99999;

        let arrow_array = ArrowKinematicEncoder::encode_kinematic(&cmd);
        lease.write_bytes(arrow_array.values()).unwrap();

        // 2. Drop-Old Mailbox
        let mailbox = DropOldMailbox::new();
        mailbox.send(lease);

        let received_lease = mailbox.recv().unwrap();
        let decoded_cmd = ArrowKinematicEncoder::decode_kinematic(&received_lease.to_arrow()).unwrap();
        assert_eq!(decoded_cmd.linear, [1.0, 2.0, 3.0]);
        assert_eq!(decoded_cmd.angular, [0.1, 0.2, 0.3]);

        // 3. Telemetri MTU Fragmenter
        let fragmenter = PacketFragmenter::new(100);
        let mut reassembler = PacketReassembler::new();
        let raw_payload = bytemuck::bytes_of(&decoded_cmd);

        let fragments = fragmenter.fragment(1, raw_payload);
        assert!(!fragments.is_empty());

        let mut completed = None;
        for frag in fragments {
            if let Some(reconstructed) = reassembler.process_fragment(frag, 100).unwrap() {
                completed = Some(reconstructed);
            }
        }
        assert_eq!(completed, Some(raw_payload.to_vec()));
    }
}
