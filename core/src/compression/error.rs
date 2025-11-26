#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("unknown compression algorithm name: {0}")]
    UnknownAlgorithm(String),

    #[error("invalid compression level value: {0}")]
    InvalidLevel(String)
}
