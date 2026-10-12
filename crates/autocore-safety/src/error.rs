//! autocore-safety :: Emniyet Katmanı Hata Tipleri

use autocore_base::SafetyMode;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum SafetyError {
    #[error("Geçersiz durum geçişi: {from:?} -> {to:?}")]
    InvalidTransition { from: SafetyMode, to: SafetyMode },

    #[error("SafeLock modu kilitli; açık sıfırlama (reset) komutu gerekiyor")]
    SafeLockActive,

    #[error("Sentinel Watchdog zaman aşımı (Heartbeat kesildi, {elapsed_us} us geçti)")]
    WatchdogTimeout { elapsed_us: u64 },

    #[error("Geofence sınır ihlali: X={x:.2}, Y={y:.2}, Z={z:.2}")]
    GeofenceBreach { x: f64, y: f64, z: f64 },

    #[error("Eylem yasak alanı (Inhibit Zone) ihlali: Görev eylemi engellendi")]
    InhibitZoneViolation,
}
