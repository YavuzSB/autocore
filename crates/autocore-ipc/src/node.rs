use tokio::sync::mpsc;
use tracing::debug;

use autocore_base::{ActuatorFeedback, KinematicCommand};
use bytemuck::{bytes_of, try_from_bytes};
use dora_node_api::dora_core::config::DataId;
use dora_node_api::{DoraNode, Event, EventStream, MetadataParameters};

use crate::error::IpcError;

/// Autocore yüksek seviyeli IPC olayları.
#[derive(Debug, Clone, PartialEq)]
pub enum AutocoreEvent {
    Kinematic(KinematicCommand),
    Feedback(ActuatorFeedback),
    Raw { id: String, data: Vec<u8> },
    InputClosed { id: String },
    Stop,
}

enum NodeBackend {
    Dora {
        node: DoraNode,
        events: EventStream,
    },
    Standalone {
        tx: mpsc::Sender<AutocoreEvent>,
        rx: mpsc::Receiver<AutocoreEvent>,
    },
}

/// Dora-rs düğüm sarmalayıcısı.
/// Hem gerçek Dora veri akış koordinatörü altında hem de bağımsız SITL test modunda çalışır.
pub struct DoraNodeWrapper {
    backend: NodeBackend,
}

impl DoraNodeWrapper {
    /// Dora ortam değişkenlerinden (`DORA_NODE_ID` vb.) gerçek Dora düğümünü başlatır.
    pub fn init_from_env() -> Result<Self, IpcError> {
        let (node, events) = DoraNode::init_from_env()
            .map_err(|e| IpcError::DoraNodeError(format!("Failed to init DoraNode from env: {e:?}")))?;

        Ok(Self {
            backend: NodeBackend::Dora { node, events },
        })
    }

    /// Dora daemon'ı olmadan masaüstü veya test ortamında koşturmak için bağımsız (Standalone) mod.
    pub fn new_standalone(buffer_size: usize) -> (Self, mpsc::Sender<AutocoreEvent>) {
        let (tx, rx) = mpsc::channel(buffer_size);
        let external_tx = tx.clone();
        (
            Self {
                backend: NodeBackend::Standalone { tx, rx },
            },
            external_tx,
        )
    }

    /// Çıktı kanalına `KinematicCommand` (32 bayt) gönderir.
    pub fn send_kinematic(&mut self, output_id: &str, cmd: &KinematicCommand) -> Result<(), IpcError> {
        let bytes = bytes_of(cmd);
        self.send_bytes(output_id, bytes)
    }

    /// Çıktı kanalına `ActuatorFeedback` (72 bayt) gönderir.
    pub fn send_feedback(&mut self, output_id: &str, fb: &ActuatorFeedback) -> Result<(), IpcError> {
        let bytes = bytes_of(fb);
        self.send_bytes(output_id, bytes)
    }

    /// Ham bayt dizisi basar.
    pub fn send_bytes(&mut self, output_id: &str, data: &[u8]) -> Result<(), IpcError> {
        match &mut self.backend {
            NodeBackend::Dora { node, .. } => {
                let data_id = DataId::from(output_id.to_string());
                let params = MetadataParameters::default();
                node.send_output_bytes(data_id, params, data.len(), data)
                    .map_err(|e| IpcError::DoraNodeError(format!("send_output_bytes failed: {e:?}")))?;
                Ok(())
            }
            NodeBackend::Standalone { tx, .. } => {
                // Standalone modda uygun olaya çevirerek kanala bas
                let event = if data.len() == std::mem::size_of::<KinematicCommand>() {
                    if let Ok(cmd) = try_from_bytes::<KinematicCommand>(data) {
                        AutocoreEvent::Kinematic(*cmd)
                    } else {
                        AutocoreEvent::Raw {
                            id: output_id.to_string(),
                            data: data.to_vec(),
                        }
                    }
                } else if data.len() == std::mem::size_of::<ActuatorFeedback>() {
                    if let Ok(fb) = try_from_bytes::<ActuatorFeedback>(data) {
                        AutocoreEvent::Feedback(*fb)
                    } else {
                        AutocoreEvent::Raw {
                            id: output_id.to_string(),
                            data: data.to_vec(),
                        }
                    }
                } else {
                    AutocoreEvent::Raw {
                        id: output_id.to_string(),
                        data: data.to_vec(),
                    }
                };

                let _ = tx.try_send(event);
                Ok(())
            }
        }
    }

    /// Asenkron olarak sıradaki IPC olayını bekler ve döner.
    pub async fn next_event(&mut self) -> Result<Option<AutocoreEvent>, IpcError> {
        match &mut self.backend {
            NodeBackend::Dora { events, .. } => {
                match events.recv_async().await {
                    Some(Event::Input { id, data, .. }) => {
                        let bytes = dora_node_api::into_vec::<u8>(&data).unwrap_or_default();
                        if bytes.len() == std::mem::size_of::<KinematicCommand>() {
                            if let Ok(cmd) = try_from_bytes::<KinematicCommand>(&bytes) {
                                return Ok(Some(AutocoreEvent::Kinematic(*cmd)));
                            }
                        } else if bytes.len() == std::mem::size_of::<ActuatorFeedback>() {
                            if let Ok(fb) = try_from_bytes::<ActuatorFeedback>(&bytes) {
                                return Ok(Some(AutocoreEvent::Feedback(*fb)));
                            }
                        }
                        Ok(Some(AutocoreEvent::Raw {
                            id: id.to_string(),
                            data: bytes,
                        }))
                    }
                    Some(Event::InputClosed { id }) => Ok(Some(AutocoreEvent::InputClosed {
                        id: id.to_string(),
                    })),
                    Some(Event::Stop(_)) => Ok(Some(AutocoreEvent::Stop)),
                    Some(other) => {
                        debug!("Dora non-input event: {other:?}");
                        Ok(None)
                    }
                    None => Ok(None),
                }
            }
            NodeBackend::Standalone { rx, .. } => Ok(rx.recv().await),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_standalone_node_wrapper_roundtrip() {
        let (mut node, _tx) = DoraNodeWrapper::new_standalone(10);

        let mut cmd = KinematicCommand::default();
        cmd.linear = [2.0, 0.0, 0.0];
        cmd.angular = [0.0, 0.0, 1.0];
        cmd.timestamp_us = 555;

        // Kinematik komut gönder
        node.send_kinematic("cmd_vel", &cmd).unwrap();

        // Olayı al
        let event = node.next_event().await.unwrap();
        assert_eq!(event, Some(AutocoreEvent::Kinematic(cmd)));
    }
}
