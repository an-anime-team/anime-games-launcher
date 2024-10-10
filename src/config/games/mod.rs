use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Games {
    pub registries: Vec<String>
}

impl Default for Games {
    fn default() -> Self {
        Self {
            registries: vec![
                String::from("https://raw.githubusercontent.com/an-anime-team/game-integrations/refs/heads/rewrite/games/registry.json")
            ]
        }
    }
}

impl AsJson for Games {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "registries": self.registries
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
                .ok_or_else(|| AsJsonError::InvalidFieldValue("games.registries"))?
        })
    }
}
