use serde::{Serialize, Deserialize};

use serde_json::Value as Json;

use anime_game_core::game::genshin::Edition;

pub mod paths;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Genshin {
    pub paths: paths::Paths,
    pub edition: Edition
}

impl From<&Json> for Genshin {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            paths: value.get("paths")
                .map(paths::Paths::from)
                .unwrap_or(default.paths),

            edition: value.get("edition")
                .and_then(|value| serde_json::from_value(value.clone()).ok())
                .unwrap_or(default.edition)
        }
    }
}
