use serde::{Serialize, Deserialize};

use serde_json::Value as Json;

pub mod prefix;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wine {
    pub build: String,
    pub version: String,
    pub prefix: prefix::Prefix
}

impl Default for Wine {
    #[inline]
    fn default() -> Self {
        Self {
            build: String::from("wine-ge-proton"),
            version: String::from("latest"),
            prefix: prefix::Prefix::default()
        }
    }
}

impl From<&Json> for Wine {
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
                .unwrap_or(default.version),

            prefix: value.get("prefix")
                .map(prefix::Prefix::from)
                .unwrap_or(default.prefix)
        }
    }
}
