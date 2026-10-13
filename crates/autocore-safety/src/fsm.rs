//! autocore-safety :: Hiyerarşik Emniyet Durum Makinesi (Central Safety FSM)

use crate::error::SafetyError;
use autocore_base::SafetyMode;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use tracing::{info, warn};

/// Atomik ve kilit-serbest (lock-free) Merkezi Emniyet Durum Makinesi
#[derive(Debug, Clone)]
pub struct CentralSafetyFsm {
    mode: Arc<AtomicU8>,
}

impl Default for CentralSafetyFsm {
    fn default() -> Self {
        Self::new(SafetyMode::Booting)
    }
}

impl CentralSafetyFsm {
    pub fn new(initial: SafetyMode) -> Self {
        Self {
            mode: Arc::new(AtomicU8::new(initial as u8)),
        }
    }

    /// Mevcut emniyet modunu atomik olarak döndürür
    pub fn current_mode(&self) -> SafetyMode {
        SafetyMode::from_u8(self.mode.load(Ordering::SeqCst)).unwrap_or(SafetyMode::SafeLock)
    }

    /// Geçerli bir durum geçişi olup olmadığını denetler
    pub fn can_transition(&self, from: SafetyMode, to: SafetyMode) -> bool {
        if from == to {
            return true;
        }

        // SafeLock modundayken hiçbir doğrudan geçiş yapılamaz (Sadece reset_safelock ile çıkılabilir)
        if from == SafetyMode::SafeLock {
            return false;
        }

        // SafeLock acil durum kilidine her durumdan anında geçilebilir
        if to == SafetyMode::SafeLock {
            return true;
        }

        match from {
            SafetyMode::Booting => matches!(to, SafetyMode::Disarmed),
            SafetyMode::Disarmed => matches!(to, SafetyMode::Armed | SafetyMode::ManualOverride),
            SafetyMode::Armed => matches!(to, SafetyMode::Disarmed | SafetyMode::Mission | SafetyMode::Failsafe | SafetyMode::ManualOverride),
            SafetyMode::Mission => matches!(to, SafetyMode::Armed | SafetyMode::Failsafe | SafetyMode::ManualOverride),
            SafetyMode::Failsafe => matches!(to, SafetyMode::Disarmed | SafetyMode::ManualOverride),
            SafetyMode::ManualOverride => matches!(to, SafetyMode::Disarmed | SafetyMode::Armed | SafetyMode::Failsafe),
            SafetyMode::SafeLock => false,
        }
    }

    /// Durum geçişi gerçekleştirir (Atomik CAS - Compare And Swap ile)
    pub fn transition(&self, target: SafetyMode) -> Result<SafetyMode, SafetyError> {
        let mut current_u8 = self.mode.load(Ordering::SeqCst);
        loop {
            let current = SafetyMode::from_u8(current_u8).unwrap_or(SafetyMode::SafeLock);
            if !self.can_transition(current, target) {
                warn!(
                    "Emniyet durumu geçişi reddedildi: {:?} -> {:?}",
                    current, target
                );
                return Err(SafetyError::InvalidTransition {
                    from: current,
                    to: target,
                });
            }

            match self.mode.compare_exchange_weak(
                current_u8,
                target as u8,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    info!("Emniyet durumu değişti: {:?} -> {:?}", current, target);
                    return Ok(target);
                }
                Err(actual) => {
                    current_u8 = actual;
                }
            }
        }
    }

    /// SafeLock modunu açıkça sıfırlar ve sistemi Disarmed moduna alır
    pub fn reset_safelock(&self) -> Result<(), SafetyError> {
        let current = self.current_mode();
        if current != SafetyMode::SafeLock {
            return Ok(());
        }

        self.mode.store(SafetyMode::Disarmed as u8, Ordering::SeqCst);
        info!("SafeLock başarıyla sıfırlandı -> DISARMED moduna geçildi");
        Ok(())
    }

    /// Sistemi anında SafeLock (acil kilit) moduna sokar (Asla hata vermez)
    pub fn trigger_safelock(&self) {
        self.mode.store(SafetyMode::SafeLock as u8, Ordering::SeqCst);
        warn!("Acil kilit tetiklendi: Sistem SAFELOCK moduna alındı!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fsm_normal_lifecycle() {
        let fsm = CentralSafetyFsm::new(SafetyMode::Booting);
        assert_eq!(fsm.current_mode(), SafetyMode::Booting);

        // Booting -> Disarmed
        assert!(fsm.transition(SafetyMode::Disarmed).is_ok());
        assert_eq!(fsm.current_mode(), SafetyMode::Disarmed);

        // Disarmed -> Armed
        assert!(fsm.transition(SafetyMode::Armed).is_ok());
        assert_eq!(fsm.current_mode(), SafetyMode::Armed);

        // Armed -> Mission
        assert!(fsm.transition(SafetyMode::Mission).is_ok());
        assert_eq!(fsm.current_mode(), SafetyMode::Mission);

        // Mission -> Armed (görev tamamlandı/duraklatıldı)
        assert!(fsm.transition(SafetyMode::Armed).is_ok());
        assert_eq!(fsm.current_mode(), SafetyMode::Armed);

        // Armed -> Disarmed
        assert!(fsm.transition(SafetyMode::Disarmed).is_ok());
        assert_eq!(fsm.current_mode(), SafetyMode::Disarmed);
    }

    #[test]
    fn test_fsm_invalid_transitions() {
        let fsm = CentralSafetyFsm::new(SafetyMode::Booting);

        // Booting -> Mission doğrudan geçilemez
        assert!(fsm.transition(SafetyMode::Mission).is_err());
        assert_eq!(fsm.current_mode(), SafetyMode::Booting);

        // Disarmed -> Mission (önce Armed olmalı)
        fsm.transition(SafetyMode::Disarmed).unwrap();
        assert!(fsm.transition(SafetyMode::Mission).is_err());
    }

    #[test]
    fn test_fsm_safelock_and_reset() {
        let fsm = CentralSafetyFsm::new(SafetyMode::Mission);

        // Herhangi bir anda SafeLock tetiklenebilir
        fsm.trigger_safelock();
        assert_eq!(fsm.current_mode(), SafetyMode::SafeLock);

        // SafeLock modundayken normal geçiş yapılamaz
        assert!(fsm.transition(SafetyMode::Armed).is_err());

        // Açık reset ile Disarmed moduna dönülebilir
        assert!(fsm.reset_safelock().is_ok());
        assert_eq!(fsm.current_mode(), SafetyMode::Disarmed);
    }
}
