use serde::{Serialize, Deserialize};

use serde_json::Value as Json;

use anime_game_core::game::GameExt;
use anime_game_core::game::genshin::Game;

use anime_game_core::game::genshin::Edition;

pub mod paths;

use crate::config;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Genshin {
    pub paths: paths::Paths,
    pub edition: Edition
}

impl Genshin {
    pub fn to_game(&self) -> Game {
        let config = config::get();

        let edition = config.games.genshin.edition;

        let driver = config.games.genshin.paths
            .for_edition(edition)
            .to_dyn_trait();

        Game::new(driver, edition)
    }
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
