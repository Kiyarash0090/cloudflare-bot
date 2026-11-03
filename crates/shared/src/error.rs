use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("KV storage error: {0}")]
    KVStorage(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Worker error: {0}")]
    Worker(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Telegram error: {0}")]
    Telegram(String),

    #[error("Bale error: {0}")]
    Bale(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<worker::Error> for Error {
    fn from(err: worker::Error) -> Self {
        Error::Worker(err.to_string())
    }
}

impl From<Error> for worker::Error {
    fn from(err: Error) -> Self {
        worker::Error::RustError(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
