use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::prelude::*;

pub mod network;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct General {
    pub language: String,
    pub network: network::Network
}

impl Default for General {
    #[inline]
    fn default() -> Self {
        Self {
            language: String::from("en-us"),
            network: network::Network::default()
        }
    }
}

impl AsJson for General {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "language": self.language,
            "network": self.network
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            language: json.get("language")
                .ok_or_else(|| AsJsonError::FieldNotFound("general.language"))?
                .as_str()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("general.language"))?
                .to_string(),

            network: json.get("network")
                .ok_or_else(|| AsJsonError::FieldNotFound("general.network"))
                .and_then(network::Network::from_json)?
        })
    }
}
