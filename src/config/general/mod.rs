use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

pub mod wine;
pub mod transitions;

use wine::Wine;
use transitions::Transitions;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct General {
    pub wine: Wine,
    pub transitions: Transitions,
    pub verify_games: bool
}

impl Default for General {
    #[inline]
    fn default() -> Self {
        Self {
            wine: Wine::default(),
            transitions: Transitions::default(),
            verify_games: true
        }
    }
}

impl From<&Json> for General {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            wine: value.get("wine")
                .map(Wine::from)
                .unwrap_or(default.wine),

            transitions: value.get("transitions")
                .map(Transitions::from)
                .unwrap_or(default.transitions),

            verify_games: value.get("verify_games")
                .and_then(Json::as_bool)
                .unwrap_or(default.verify_games)
        }
    }
}
