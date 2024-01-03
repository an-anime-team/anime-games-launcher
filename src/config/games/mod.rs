use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::LAUNCHER_FOLDER;

use crate::games::integrations;
use crate::config::driver::Driver;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Games {
    pub integrations: Driver,
    pub settings: HashMap<String, GameSettings>
}

impl Default for Games {
    #[inline]
    fn default() -> Self {
        Self {
            integrations: Driver::PhysicalFsDriver {
                base_folder: LAUNCHER_FOLDER.join("integrations")
            },

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
                .and_then(|value| serde_json::from_value(value.clone()).ok())
                .unwrap_or(default.integrations),

            settings: match value.get("settings").and_then(Json::as_object) {
                Some(values) => {
                    let mut settings = HashMap::new();

                    for (name, game) in values {
                        settings.insert(name.to_owned(), GameSettings::from(game));
                    }

                    settings
                }

                None => HashMap::new()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSettings {
    pub paths: HashMap<String, Driver>
}

impl GameSettings {
    pub fn default_for_game(game: &integrations::Game) -> anyhow::Result<Self> {
        let editions = game.get_game_editions_list()?;

        Ok(Self {
            paths: editions.into_iter().map(|edition| {
                (edition.name, Driver::PhysicalFsDriver {
                    base_folder: LAUNCHER_FOLDER
                        .join("games")
                        .join(&game.game_title)
                        .join(edition.title)
                })
            }).collect()
        })
    }
}

impl From<&Json> for GameSettings {
    #[inline]
    fn from(value: &Json) -> Self {
        Self {
            paths: match value.get("paths").and_then(Json::as_object) {
                Some(values) => {
                    let mut paths = HashMap::new();

                    for (name, path) in values.clone() {
                        paths.insert(name, serde_json::from_value(path).unwrap());
                    }

                    paths
                }

                None => HashMap::new()
            }
        }
    }
}
