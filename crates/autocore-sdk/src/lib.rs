//! autocore-sdk :: Autocore İstemci Kiti ve Geliştirici SDK'sı
//!
//! Otonom robotik ve savunma sistemleri için yüksek seviyeli istemci kiti,
//! dairesel ikili siyah kutu (Blackbox) kaydedici ve çok dilli entegrasyon araçları.

pub mod blackbox;
pub mod client;

pub use blackbox::{
    BlackboxError, BlackboxFrame, BlackboxHeader, BlackboxReader, BlackboxRecorder,
    BLACKBOX_MAGIC, BLACKBOX_VERSION,
};
pub use client::AutocoreClient;

// Çekirdek tipleri yeniden dışa aktar
pub use autocore_base::{
    ActuatorFeedback, KinematicCommand, SafetyMode, StateVector6DoF,
};
