//! autocore-hal :: Donanımsal Emniyet Pinleri ve Kesme Yönetimi (GPIO Safety)

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::warn;

/// Donanımsal Acil Durdurma (E-Stop) Pini Simülasyonu ve Soyutlaması
#[derive(Debug, Clone)]
pub struct GpioSafetyPin {
    pin_number: u32,
    active_low: bool,
    is_pressed: Arc<AtomicBool>,
}

impl GpioSafetyPin {
    pub fn new(pin_number: u32, active_low: bool) -> Self {
        Self {
            pin_number,
            active_low,
            is_pressed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Fiziksel veya yazılımsal buton basılma durumunu simüle eder/günceller
    pub fn set_state(&self, pressed: bool) {
        self.is_pressed.store(pressed, Ordering::SeqCst);
        if pressed {
            warn!("GPIO Pin #{}: Acil durdurma (E-Stop) tetiklendi!", self.pin_number);
        }
    }

    /// E-Stop butonunun basılı olup olmadığını döndürür
    pub fn is_estop_triggered(&self) -> bool {
        self.is_pressed.load(Ordering::SeqCst)
    }

    pub fn pin_number(&self) -> u32 {
        self.pin_number
    }

    pub fn is_active_low(&self) -> bool {
        self.active_low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpio_pin_toggle() {
        let pin = GpioSafetyPin::new(17, true);
        assert!(!pin.is_estop_triggered());

        pin.set_state(true);
        assert!(pin.is_estop_triggered());

        pin.set_state(false);
        assert!(!pin.is_estop_triggered());
    }
}
