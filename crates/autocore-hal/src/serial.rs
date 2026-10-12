//! autocore-hal :: Asenkron Seri / Akış Sürücüsü (Async Stream Actuator)

use crate::protocol::{
    decode_packet, encode_packet, MSG_TYPE_CMD_VEL, MSG_TYPE_ESTOP, MSG_TYPE_FEEDBACK,
};
use crate::traits::ActuatorPort;
use async_trait::async_trait;
use autocore_base::{ActuatorFeedback, HalError, KinematicCommand};
use bytemuck::try_from_bytes;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tracing::{debug, error, info};

/// Herhangi bir asenkron I/O akışı (UART, PTY, TCP vb.) üzerinden çalışan aktüatör sürücüsü
pub struct AsyncStreamActuator<S> {
    stream: S,
    read_buffer: Vec<u8>,
    connected: bool,
    latest_feedback: ActuatorFeedback,
}

impl<S> AsyncStreamActuator<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            read_buffer: Vec::with_capacity(1024),
            connected: true,
            latest_feedback: ActuatorFeedback::default(),
        }
    }

    /// Gelen bayt akışını okur ve geri besleme paketlerini ayrıştırır
    async fn poll_incoming(&mut self) -> Result<(), HalError> {
        let mut temp_buf = [0u8; 256];
        match self.stream.read(&mut temp_buf).await {
            Ok(0) => {
                self.connected = false;
                return Err(HalError::SerialError("Bağlantı koptu (EOF)".into()));
            }
            Ok(n) => {
                self.read_buffer.extend_from_slice(&temp_buf[..n]);
            }
            Err(e) => {
                self.connected = false;
                return Err(HalError::SerialError(e.to_string()));
            }
        }

        // Paketi ayrıştır
        while let Some((msg_type, payload, consumed)) = decode_packet(&self.read_buffer)? {
            self.read_buffer.drain(..consumed);

            if msg_type == MSG_TYPE_FEEDBACK {
                if let Ok(feedback) = try_from_bytes::<ActuatorFeedback>(&payload) {
                    self.latest_feedback = *feedback;
                    debug!("Aktüatör geri beslemesi alındı: {:?}", feedback);
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<S> ActuatorPort for AsyncStreamActuator<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    async fn send_command(&mut self, cmd: &KinematicCommand) -> Result<(), HalError> {
        if !self.connected {
            return Err(HalError::SerialError("Port bağlı değil".into()));
        }

        let packet = encode_packet(MSG_TYPE_CMD_VEL, cmd.as_bytes());
        self.stream
            .write_all(&packet)
            .await
            .map_err(|e| HalError::SerialError(e.to_string()))?;
        self.stream
            .flush()
            .await
            .map_err(|e| HalError::SerialError(e.to_string()))?;

        Ok(())
    }

    async fn read_feedback(&mut self) -> Result<ActuatorFeedback, HalError> {
        // Varsa yeni gelen verileri tara
        let _ = self.poll_incoming().await;
        Ok(self.latest_feedback)
    }

    async fn emergency_stop(&mut self) -> Result<(), HalError> {
        info!("Donanıma ACİL DURDURMA (E-Stop) paketi gönderiliyor!");
        let packet = encode_packet(MSG_TYPE_ESTOP, &[0xFF]);
        if let Err(e) = self.stream.write_all(&packet).await {
            error!("E-Stop paketi gönderilemedi: {}", e);
            return Err(HalError::SerialError(e.to_string()));
        }
        let _ = self.stream.flush().await;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[tokio::test]
    async fn test_async_stream_actuator_send() {
        let (client, mut server) = duplex(1024);
        let mut actuator = AsyncStreamActuator::new(client);

        let cmd = KinematicCommand::new([1.0, 2.0, 0.0], [0.0, 0.0, 0.5], 100);
        actuator.send_command(&cmd).await.unwrap();

        // Server tarafından oku ve doğrula
        let mut buf = [0u8; 128];
        let n = server.read(&mut buf).await.unwrap();
        let decoded = decode_packet(&buf[..n]).unwrap().unwrap();

        assert_eq!(decoded.0, MSG_TYPE_CMD_VEL);
        let recv_cmd = try_from_bytes::<KinematicCommand>(&decoded.1).unwrap();
        assert_eq!(recv_cmd, &cmd);
    }
}
