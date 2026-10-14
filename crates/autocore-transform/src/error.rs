//! autocore-transform :: Hata Tipleri

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TransformError {
    #[error("Koordinat çerçevesi bulunamadı: '{0}'")]
    FrameNotFound(String),

    #[error("Koordinat ağacında döngüsel bağımlılık tespit edildi: '{0}'")]
    CyclicDependency(String),

    #[error("Ortak ata koordinat çerçevesi bulunamadı: '{from}' -> '{to}'")]
    NoCommonParent { from: String, to: String },

    #[error("Geçersiz veya normalize edilemeyen kuaterniyon (Norm sıfır)")]
    InvalidQuaternion,

    #[error("Sayısal hesaplama hatası veya sınır dışı koordinat: {0}")]
    NumericalError(String),
}
