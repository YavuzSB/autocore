//! autocore-base :: Ortak Hata Tipleri

use thiserror::Error;

/// Autocore çekirdek genel hata tipi
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AutocoreError {
    #[error("C-ABI hizalama hatası: beklenen {expected} bayt, alınan {actual} bayt")]
    AbiAlignmentError { expected: usize, actual: usize },

    #[error("Geçersiz veri paketi boyutu: {0} bayt")]
    InvalidPayloadSize(usize),

    #[error("Geçersiz zaman damgası: {0} us")]
    InvalidTimestamp(u64),

    #[error("Doğrulama başarısız: {0}")]
    ValidationFailed(String),

    #[error("Bilinmeyen veya geçersiz emniyet modu kodu: {0}")]
    InvalidSafetyMode(u8),
}

/// Donanım ve HAL soyutlama hataları
#[derive(Debug, Error, PartialEq, Eq)]
pub enum HalError {
    #[error("Donanım portu açılamadı veya bulunamadı: {0}")]
    PortNotFound(String),

    #[error("Seri haberleşme / UART hatası: {0}")]
    SerialError(String),

    #[error("CAN Bus / SocketCAN hatası: {0}")]
    CanError(String),

    #[error("GPIO kesme / pin hatası: {0}")]
    GpioError(String),

    #[error("Donanım zaman aşımı (Timeout): {0} ms")]
    Timeout(u64),

    #[error("Acil durdurma (E-Stop) aktif; donanım kilitlendi")]
    EmergencyStopActive,

    #[error("Donanım CRC / sağlama toplamı hatası")]
    ChecksumMismatch,
}
