use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::core::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct General {
    pub verify_games: bool
}

impl Default for General {
    fn default() -> Self {
        Self {
            verify_games: true
        }
    }
}

impl AsJson for General {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "verify_games": self.verify_games
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            verify_games: json.get("verify_games")
                .ok_or_else(|| AsJsonError::FieldNotFound("general.verify_games"))?
                .as_bool()
                .ok_or_else(|| AsJsonError::InvalidFieldValue("general.verify_games"))?
        })
    }
}
