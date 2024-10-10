use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::prelude::*;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Games {
    /// List of URLs to the games manifests' registries.
    pub registries: Vec<String>,

    /// Timeout of the manifests fetching, in seconds.
    pub fetch_timeout: u64
}

impl AsJson for Games {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "registries": self.registries,
            "fetch_timeout": self.fetch_timeout
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            registries: json.get("registries")
                .ok_or_else(|| AsJsonError::FieldNotFound("games.registries"))?
                .as_array()
                .and_then(|registries| {
                    registries.iter()
                        .map(|url| url.as_str().map(String::from))
                        .collect::<Option<Vec<_>>>()
                })
                .ok_or_else(|| AsJsonError::InvalidFieldValue("games.registries"))?,

            fetch_timeout: json.get("fetch_timeout")
                .ok_or_else(|| AsJsonError::FieldNotFound("games.fetch_timeout"))?
                .as_u64()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("games.fetch_timeout"))?
        })
    }
}
