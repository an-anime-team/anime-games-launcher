use serde::{Serialize, Deserialize};

use serde_json::Value as Json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dxvk {
    pub build: String,
    pub version: String
}

impl Default for Dxvk {
    #[inline]
    fn default() -> Self {
        Self {
            build: String::from("gplasync"),
            version: String::from("latest")
        }
    }
}

impl From<&Json> for Dxvk {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            build: value.get("build")
                .and_then(Json::as_str)
                .map(String::from)
                .unwrap_or(default.build),

            version: value.get("version")
                .and_then(Json::as_str)
                .map(String::from)
                .unwrap_or(default.version)
        }
    }
}
