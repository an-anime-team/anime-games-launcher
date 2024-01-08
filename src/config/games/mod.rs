use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::config;
use crate::games::integrations::Game;

use crate::LAUNCHER_FOLDER;

pub mod settings;

use settings::GameSettings;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Games {
    pub integrations: PathBuf,
    settings: Json
}

impl Default for Games {
    #[inline]
    fn default() -> Self {
        Self {
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
