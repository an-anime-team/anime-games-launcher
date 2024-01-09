use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::config;
use crate::games::integrations::Game;

pub mod wine;
pub mod enhancements;
pub mod integrations;
pub mod settings;

pub mod prelude {
    pub use super::wine::prelude::*;
    pub use super::enhancements::prelude::*;
    pub use super::settings::prelude::*;

    pub use super::integrations::Integrations;

    pub use super::Games;
}

use prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Games {
    pub wine: Wine,
    pub enhancements: Enhancements,
    pub environment: HashMap<String, String>,
    pub integrations: Integrations,

    settings: Json
}

impl Default for Games {
    #[inline]
    fn default() -> Self {
        Self {
            wine: Wine::default(),
            enhancements: Enhancements::default(),
            environment: HashMap::new(),
            integrations: Integrations::default(),
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
                .map(Integrations::from)
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

        let settings = match self.settings.get(&game.manifest.game_name) {
            Some(settings) => GameSettings::from_json(&game.manifest.game_name, editions, settings)?,
            None => GameSettings::default(&game.manifest.game_name, editions)?
        };

        config::set(format!("games.settings.{}", game.manifest.game_name), serde_json::to_value(settings.clone())?)?;

        Ok(settings)
    }
}
