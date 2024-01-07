use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::games;
use crate::config;

use crate::LAUNCHER_FOLDER;

pub mod settings;

use settings::GameSettings;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Games {
    pub integrations: PathBuf,
    pub settings: HashMap<String, GameSettings>
}

impl Default for Games {
    #[inline]
    fn default() -> Self {
        Self {
            integrations: LAUNCHER_FOLDER.join("integrations"),
            settings: HashMap::new()
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
                .and_then(Json::as_object)
                .map(|values| {
                    let mut settings = HashMap::new();

                    for (name, game) in values {
                        let editions: &[&str] = &[];

                        if let Ok(game_settings) = GameSettings::from_json(name, editions, game) {
                            settings.insert(name.to_owned(), game_settings);
                        }
                    }

                    settings
                })
                .unwrap_or(default.settings)
        }
    }
}

impl Games {
    pub fn get_game_settings(&self, game: impl AsRef<str>) -> anyhow::Result<GameSettings> {
        match self.settings.get(game.as_ref()) {
            Some(settings) => Ok(settings.to_owned()),
            None => {
                let Some(game_object) = games::get(game.as_ref())? else {
                    anyhow::bail!("Couldn't find {} integration script", game.as_ref());
                };

                let editions = game_object
                    .get_game_editions_list()?
                    .into_iter()
                    .map(|edition| edition.name);

                let settings = GameSettings::default(&game_object.game_name, editions)?;

                config::set(format!("games.settings.{}", game.as_ref()), serde_json::to_value(settings.clone())?)?;

                Ok(settings)
            }
        }
    }
}
