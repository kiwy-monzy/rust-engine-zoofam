use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("signing material: {0}")]
    Keys(String),
    #[error("pass build/serialize: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("openssl: {0}")]
    OpenSsl(#[from] openssl::error::ErrorStack),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("unknown sample pass: {0}")]
    UnknownSample(String),
    #[error("unknown serial: {0}")]
    UnknownSerial(String),
    #[error("apns: {0}")]
    Apns(String),
    #[error("qrcode: {0}")]
    Qr(String),
}

pub type WalletResult<T> = Result<T, WalletError>;
