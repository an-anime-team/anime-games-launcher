use serde_json::Value as Json;

#[derive(Debug, thiserror::Error)]
pub enum AsJsonError {
    #[error("Field not found: {0}")]
    FieldNotFound(&'static str),

    #[error("Invalid field value: {0}")]
    InvalidFieldValue(&'static str),

    #[error("Unsupported format version: {0}")]
    UnsupportedFormat(u64),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>)
}

pub trait AsJson {
    fn to_json(&self) -> Result<Json, AsJsonError>;
    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized;
}
