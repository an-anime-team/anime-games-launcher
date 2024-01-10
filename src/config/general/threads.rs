use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Threads {
    pub number: u64
}

impl Default for Threads {
    #[inline]
    fn default() -> Self {
        Self {
            number: 0
        }
    }
}

impl From<&Json> for Threads {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            number: value.get("number")
                .and_then(Json::as_u64)
                .unwrap_or(default.number)
        }
    }
}
