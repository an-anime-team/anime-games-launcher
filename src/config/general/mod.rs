use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

pub mod transitions;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct General {
    pub verify_games: bool,
    pub transitions: transitions::Transitions
}

impl Default for General {
    #[inline]
    fn default() -> Self {
        Self {
            verify_games: true,
            transitions: transitions::Transitions::default()
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

            transitions: value.get("transitions")
                .map(transitions::Transitions::from)
                .unwrap_or(default.transitions)
        }
    }
}
