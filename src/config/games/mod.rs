use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::config;
use crate::games::integrations::Game;

use crate::LAUNCHER_FOLDER;

pub mod wine;
pub mod enhancements;
pub mod settings;

pub mod prelude {
    pub use super::wine::prelude::*;
    pub use super::enhancements::prelude::*;
    pub use super::settings::prelude::*;

    pub use super::Games;
}

use prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Games {
    pub wine: Wine,
    pub enhancements: Enhancements,
    pub environment: HashMap<String, String>,
    pub integrations: PathBuf,

    settings: Json
}

impl Default for Games {
    #[inline]
    fn default() -> Self {
        Self {
            wine: Wine::default(),
            enhancements: Enhancements::default(),
            environment: HashMap::new(),
            integrations: LAUNCHER_FOLDER.join("integrations"),
            settings: Json::Object(serde_json::Map::default())
        }
    }
}

impl From<&Json> for Games {
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

            integrations: value.get("integrations")
                .and_then(Json::as_str)
                .map(PathBuf::from)
                .unwrap_or(default.integrations),

            settings: value.get("settings")
                .cloned()
                .unwrap_or(default.settings)
        }
    }
}

impl Games {
    pub fn get_game_settings(&self, game: &Game) -> anyhow::Result<GameSettings> {
        let editions = game
            .get_game_editions_list()?
            .into_iter()
            .map(|edition| edition.name);

        let settings = match self.settings.get(&game.game_name) {
            Some(settings) => GameSettings::from_json(&game.game_name, editions, settings)?,
            None => GameSettings::default(&game.game_name, editions)?
        };

        config::set(format!("games.settings.{}", game.game_name), serde_json::to_value(settings.clone())?)?;

        Ok(settings)
    }
}
