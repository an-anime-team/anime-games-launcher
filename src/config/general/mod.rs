use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

pub mod wine;
pub mod enhancements;
pub mod transitions;

pub mod prelude {
    pub use super::wine::prelude::*;
    pub use super::enhancements::prelude::*;

    pub use super::transitions::Transitions;

    pub use super::General;
}

use prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct General {
    pub wine: Wine,
    pub enhancements: Enhancements,
    pub environment: HashMap<String, String>,
    pub transitions: Transitions,
    pub verify_games: bool
}

impl Default for General {
    #[inline]
    fn default() -> Self {
        Self {
            wine: Wine::default(),
            enhancements: Enhancements::default(),
            environment: HashMap::new(),
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

            enhancements: value.get("enhancements")
                .map(Enhancements::from)
                .unwrap_or(default.enhancements),

            environment: value.get("environment")
                .and_then(Json::as_object)
                .map(|object| object.into_iter()
                    .filter_map(|(key, value)| {
                        value.as_str().map(|value| (key.to_string(), value.to_string()))
                    })
                    .collect::<HashMap<_, _>>()
                )
                .unwrap_or(default.environment),

            transitions: value.get("transitions")
                .map(Transitions::from)
                .unwrap_or(default.transitions),

            verify_games: value.get("verify_games")
                .and_then(Json::as_bool)
                .unwrap_or(default.verify_games)
        }
    }
}
