//! autocore-sitl :: Sanal PTY ile Mock Aktüatör Simülasyon Köprüsü (SITL Bridge)

use crate::mock_actuator::MockActuator;
use crate::pty::VirtualPtyPair;
use autocore_base::{HalError, KinematicCommand};
use autocore_hal::{
    decode_packet, encode_packet, ActuatorPort, MSG_TYPE_CMD_VEL, MSG_TYPE_ESTOP, MSG_TYPE_FEEDBACK,
};
use bytemuck::try_from_bytes;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

/// Sanal PTY üzerinden çalışan tam donanım emülasyon köprüsü
pub struct SitlSerialBridge {
    pty: VirtualPtyPair,
    mock_actuator: MockActuator,
    running: Arc<AtomicBool>,
    worker_handles: Vec<JoinHandle<()>>,
}

impl SitlSerialBridge {
    /// Köprüyü açar ve arka plan simülasyon görevini başlatır
    pub fn start() -> Result<Self, HalError> {
        let pty = VirtualPtyPair::open()?;
        let mock_actuator = MockActuator::default();
        let running = Arc::new(AtomicBool::new(true));

        let mut master_read = pty.master_tokio_file()?;
        let mut master_write = pty.master_tokio_file()?;

        let actuator_reader = mock_actuator.clone();
        let running_reader = running.clone();

        // 1. Görev: Master PTY'den gelen komutları oku ve mock aktüatöre ilet
        let reader_handle = tokio::spawn(async move {
            let mut read_buf = Vec::with_capacity(1024);
            let mut temp_buf = [0u8; 256];

            while running_reader.load(Ordering::SeqCst) {
                match master_read.read(&mut temp_buf).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        read_buf.extend_from_slice(&temp_buf[..n]);
                        while let Ok(Some((msg_type, payload, consumed))) = decode_packet(&read_buf) {
                            read_buf.drain(..consumed);
                            match msg_type {
                                MSG_TYPE_CMD_VEL => {
                                    if let Ok(cmd) = try_from_bytes::<KinematicCommand>(&payload) {
                                        let mut act = actuator_reader.clone();
                                        let _ = act.send_command(cmd).await;
                                        debug!("SITL Bridge: Komut aktüatöre iletildi: {:?}", cmd);
                                    }
                                }
                                MSG_TYPE_ESTOP => {
                                    let mut act = actuator_reader.clone();
                                    let _ = act.emergency_stop().await;
                                    info!("SITL Bridge: E-Stop tetiklendi!");
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        error!("SITL Bridge master okuma hatası: {}", e);
                        break;
                    }
                }
            }
        });

        let mut actuator_writer = mock_actuator.clone();
        let running_writer = running.clone();

        // 2. Görev: 50 Hz frekansta telemetri/geri besleme bas
        let writer_handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(20)); // 50 Hz

            while running_writer.load(Ordering::SeqCst) {
                interval.tick().await;
                if let Ok(feedback) = actuator_writer.read_feedback().await {
                    let packet = encode_packet(MSG_TYPE_FEEDBACK, feedback.as_bytes());
                    if let Err(e) = master_write.write_all(&packet).await {
                        error!("SITL Bridge telemetri yazma hatası: {}", e);
                        break;
                    }
                    let _ = master_write.flush().await;
                }
            }
        });

        Ok(Self {
            pty,
            mock_actuator,
            running,
            worker_handles: vec![reader_handle, writer_handle],
        })
    }

    /// Robot kontrolcüsünün bağlanacağı sanal seri port yolunu döndürür (Örn: `/dev/pts/2`)
    pub fn serial_port_path(&self) -> &std::path::Path {
        self.pty.slave_path()
    }

    /// Mock aktüatör referansını döndürür
    pub fn actuator(&self) -> &MockActuator {
        &self.mock_actuator
    }

    /// Köprüyü durdurur
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        for h in self.worker_handles.drain(..) {
            h.abort();
        }
    }
}

impl Drop for SitlSerialBridge {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autocore_hal::AsyncStreamActuator;
    use std::time::Duration;

    #[tokio::test]
    async fn test_sitl_bridge_end_to_end() {
        // 1. Köprüyü başlat
        let bridge = SitlSerialBridge::start().expect("SITL Bridge başlatılamadı");
        let slave_file = bridge.pty.open_slave_tokio().expect("Slave açılamadı");

        // 2. Kontrolcü tarafında AsyncStreamActuator bağla
        let mut client_actuator = AsyncStreamActuator::new(slave_file);

        // 3. Komut gönder
        let cmd = KinematicCommand::new([5.0, 0.0, 0.0], [0.0; 3], 1_000);
        client_actuator.send_command(&cmd).await.expect("Komut gönderilemedi");

        // 4. Köprünün komutu işleyip geri besleme üretmesini bekle
        tokio::time::sleep(Duration::from_millis(100)).await;

        let feedback = client_actuator.read_feedback().await.expect("Geri besleme okunamadı");
        // Mock aktüatör hızlanmaya başlamış olmalı
        assert!(feedback.actual_linear[0] > 0.1);
    }
}
