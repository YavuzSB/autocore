//! autocore-safety :: Gerçek Zamanlı Sentinel Watchdog

use crate::error::SafetyError;
use crate::fsm::CentralSafetyFsm;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::{error, trace};

/// 500 ms (500.000 us) varsayılan eşikli Emniyet Bekçisi
#[derive(Debug)]
pub struct SentinelWatchdog {
    last_heartbeat_us: AtomicU64,
    timeout_threshold_us: u64,
}

impl Default for SentinelWatchdog {
    fn default() -> Self {
        Self::new(500_000) // 500 ms
    }
}

impl SentinelWatchdog {
    pub fn new(timeout_threshold_us: u64) -> Self {
        Self {
            last_heartbeat_us: AtomicU64::new(0),
            timeout_threshold_us,
        }
    }

    /// Watchdog'u taze kalp atışı ile besler
    pub fn feed(&self, now_us: u64) {
        self.last_heartbeat_us.store(now_us, Ordering::SeqCst);
        trace!("Sentinel Watchdog beslendi: {} us", now_us);
    }

    /// Son besleme zaman damgasını döndürür
    pub fn last_heartbeat(&self) -> u64 {
        self.last_heartbeat_us.load(Ordering::SeqCst)
    }

    /// Zaman aşımı eşiğini mikrosaniye cinsinden döndürür
    pub fn timeout_threshold_us(&self) -> u64 {
        self.timeout_threshold_us
    }

    /// Kalp atışını denetler; eşik aşıldıysa durum makinesini SafeLock moduna sokar
    pub fn check_timeout(
        &self,
        now_us: u64,
        fsm: &CentralSafetyFsm,
    ) -> Result<(), SafetyError> {
        let last = self.last_heartbeat_us.load(Ordering::SeqCst);
        if last == 0 {
            // Henüz ilk kalp atışı gelmemişse denetim pas geçilir
            return Ok(());
        }

        if now_us > last {
            let elapsed = now_us - last;
            if elapsed > self.timeout_threshold_us {
                error!(
                    "WATCHDOG ZAMAN AŞIMI! Son kalp atışından bu yana {} us (> {} us) geçti!",
                    elapsed, self.timeout_threshold_us
                );
                fsm.trigger_safelock();
                return Err(SafetyError::WatchdogTimeout {
                    elapsed_us: elapsed,
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use autocore_base::SafetyMode;

    #[test]
    fn test_watchdog_healthy_feed() {
        let wd = SentinelWatchdog::new(500_000);
        let fsm = CentralSafetyFsm::new(SafetyMode::Mission);

        wd.feed(1_000_000);
        assert_eq!(wd.last_heartbeat(), 1_000_000);

        // 300 ms sonra kontrol -> Sağlıklı
        assert!(wd.check_timeout(1_300_000, &fsm).is_ok());
        assert_eq!(fsm.current_mode(), SafetyMode::Mission);
    }

    #[test]
    fn test_watchdog_timeout_triggers_safelock() {
        let wd = SentinelWatchdog::new(500_000);
        let fsm = CentralSafetyFsm::new(SafetyMode::Mission);

        wd.feed(1_000_000);

        // 600 ms sonra kontrol (500 ms eşiğini aştı) -> SafeLock tetiklenmeli
        let res = wd.check_timeout(1_600_001, &fsm);
        assert!(res.is_err());
        assert_eq!(fsm.current_mode(), SafetyMode::SafeLock);
    }
}
