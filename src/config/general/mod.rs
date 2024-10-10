use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::prelude::*;

pub mod network;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct General {
    pub network: network::Network
}

impl AsJson for General {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "network": self.network
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            network: json.get("network")
                .ok_or_else(|| AsJsonError::FieldNotFound("general.network"))
                .and_then(network::Network::from_json)?
        })
    }
}
