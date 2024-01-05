use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::games;
use crate::games::integrations;

use crate::config;

use crate::LAUNCHER_FOLDER;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSettings {
    pub paths: HashMap<String, GameEditionPaths>,
    pub addons: HashMap<String, Vec<GameEditionAddon>>
}

impl GameSettings {
    pub fn default(game_name: impl AsRef<str>, edition_names: impl IntoIterator<Item = impl AsRef<str>>) -> anyhow::Result<Self> {
        Ok(Self {
            paths: edition_names
                .into_iter()
                .filter_map(|edition| {
                    match GameEditionPaths::default(game_name.as_ref(), edition.as_ref()) {
                        Ok(paths) => Some((edition.as_ref().to_string(), paths)),
                        Err(_) => None
                    }
                }).collect(),

            addons: HashMap::new()
        })
    }

    pub fn from_json(game_name: impl AsRef<str>, edition_names: impl IntoIterator<Item = impl AsRef<str>>, value: &Json) -> anyhow::Result<Self> {
        let default = Self::default(game_name.as_ref(), edition_names)?;

        Ok(Self {
            paths: value.get("paths")
                .and_then(Json::as_object)
                .map(|values| {
                    let mut paths = HashMap::new();

                    for (edition, edition_paths) in values {
                        if let Ok(value) = GameEditionPaths::from_json(game_name.as_ref(), edition, edition_paths) {
                            paths.insert(edition.to_owned(), value);
                        }
                    }

                    paths
                })
                .unwrap_or(default.paths),

            addons: value.get("addons")
                .and_then(Json::as_object)
                .map(|values| {
                    let mut addons = HashMap::new();

                    for (edition, names) in values.clone() {
                        if let Some(names) = names.as_array() {
                            addons.insert(edition, names.iter().map(GameEditionAddon::from).collect());
                        }
                    }

                    addons
                })
                .unwrap_or(default.addons)
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameEditionAddon {
    pub group: String,
    pub name: String
}

impl From<&Json> for GameEditionAddon {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            group: value.get("group")
                .and_then(Json::as_str)
                .map(String::from)
                .unwrap_or(default.group),

            name: value.get("name")
                .and_then(Json::as_str)
                .map(String::from)
                .unwrap_or(default.name)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameEditionPaths {
    pub game: PathBuf,
    pub addons: PathBuf
}

impl GameEditionPaths {
    pub fn default(game_name: impl AsRef<str>, edition_name: impl AsRef<str>) -> anyhow::Result<Self> {
        Ok(Self {
            game: LAUNCHER_FOLDER
                .join("games")
                .join(game_name.as_ref())
                .join(edition_name.as_ref())
                .join("game"),

            addons: LAUNCHER_FOLDER
                .join("games")
                .join(game_name.as_ref())
                .join(edition_name.as_ref())
                .join("addons")
        })
    }

    pub fn from_json(game_name: impl AsRef<str>, edition_name: impl AsRef<str>, value: &Json) -> anyhow::Result<Self> {
        let default = Self::default(game_name, edition_name)?;

        Ok(Self {
            game: value.get("game")
                .and_then(Json::as_str)
                .map(PathBuf::from)
                .unwrap_or(default.game),

            addons: value.get("addons")
                .and_then(Json::as_str)
                .map(PathBuf::from)
                .unwrap_or(default.addons)
        })
    }
}
