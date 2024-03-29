use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::i18n;

pub mod transitions;
pub mod threads;

pub mod prelude {
    pub use super::transitions::Transitions;
    pub use super::threads::Threads;
    pub use super::General;
}

use prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct General {
    pub transitions: Transitions,
    pub threads: Threads,
    pub language: String,
    pub verify_games: bool
}

impl Default for General {
    #[inline]
    fn default() -> Self {
        Self {
            transitions: Transitions::default(),
            threads: Threads::default(),
            language: i18n::format_language(&i18n::get_default_language()),
            verify_games: true
        }
    }
}

impl From<&Json> for General {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            transitions: value.get("transitions")
                .map(Transitions::from)
                .unwrap_or(default.transitions),

            threads: value.get("threads")
                .map(Threads::from)
                .unwrap_or(default.threads),

            language: value.get("language")
                .and_then(Json::as_str)
                .map(String::from)
                .unwrap_or(default.language),

            verify_games: value.get("verify_games")
                .and_then(Json::as_bool)
                .unwrap_or(default.verify_games)
        }
    }
}
