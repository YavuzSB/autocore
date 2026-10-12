//! autocore-sitl :: Sanal CAN (vcan) Döngü ve Simülasyon Katmanı

use autocore_base::HalError;
use autocore_hal::CanFrame;
use tokio::sync::mpsc::{channel, Receiver, Sender};

/// Sanal CAN (vcan) döngü hattı simülatörü
pub struct VirtualCanBus {
    tx: Sender<CanFrame>,
    rx: Receiver<CanFrame>,
}

impl VirtualCanBus {
    pub fn new(capacity: usize) -> (Self, VirtualCanEndpoint) {
        let (bus_tx, client_rx) = channel(capacity);
        let (client_tx, bus_rx) = channel(capacity);

        let bus = Self {
            tx: bus_tx,
            rx: bus_rx,
        };

        let endpoint = VirtualCanEndpoint {
            tx: client_tx,
            rx: client_rx,
        };

        (bus, endpoint)
    }

    /// Veri yoluna gelen çerçeveyi dinler
    pub async fn recv_frame(&mut self) -> Option<CanFrame> {
        self.rx.recv().await
    }

    /// Veri yoluna çerçeve yazar
    pub async fn send_frame(&self, frame: CanFrame) -> Result<(), HalError> {
        self.tx
            .send(frame)
            .await
            .map_err(|e| HalError::CanError(e.to_string()))
    }
}

pub struct VirtualCanEndpoint {
    tx: Sender<CanFrame>,
    rx: Receiver<CanFrame>,
}

impl VirtualCanEndpoint {
    pub async fn send_frame(&self, frame: CanFrame) -> Result<(), HalError> {
        self.tx
            .send(frame)
            .await
            .map_err(|e| HalError::CanError(e.to_string()))
    }

    pub async fn recv_frame(&mut self) -> Option<CanFrame> {
        self.rx.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_virtual_can_loopback() {
        let (mut bus, endpoint) = VirtualCanBus::new(16);

        let frame = CanFrame::new_standard(0x100, &[1, 2, 3, 4]);
        endpoint.send_frame(frame).await.unwrap();

        let recv_frame = bus.recv_frame().await.unwrap();
        assert_eq!(recv_frame, frame);
    }
}
