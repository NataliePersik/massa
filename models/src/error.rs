use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModelsError {
    #[error("hashing error")]
    HashError,
    #[error("Derialization error:{0}")]
    SerializeError(String),
    #[error("Derialization error:{0}")]
    DeserializeError(String),
    #[error("buffer error: {0}")]
    BufferError(String),
    #[error("crypto error: {0}")]
    CryptoError(#[from] crypto::CryptoError),
    #[error("thread overflow error")]
    ThreadOverflowError,
    #[error("period overflow error")]
    PeriodOverflowError,
}