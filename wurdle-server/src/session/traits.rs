use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Compression(#[from] flate2::CompressError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Signing(#[from] ring::error::Unspecified),
    #[error("invalid format")]
    InvalidFormatting,
}
