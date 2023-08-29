use serde::{Serialize, Deserialize};

use serde_json::Value as Json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct General {
    pub verify_games: bool
}

impl Default for General {
    #[inline]
    fn default() -> Self {
        Self {
            verify_games: true
        }
    }
}

impl From<&Json> for General {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            verify_games: value.get("verify_games")
                .and_then(Json::as_bool)
                .unwrap_or(default.verify_games),
        }
    }
}
