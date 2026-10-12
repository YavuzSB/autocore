//! autocore-base :: Yüksek Çözünürlüklü Zamanlayıcı Yardımcıları

use std::time::{SystemTime, UNIX_EPOCH};

/// Mevcut UNIX zaman damgasını mikrosaniye (us) olarak döndürür
pub fn now_us() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}
