use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

pub mod edition_addons;
pub mod edition_paths;

use edition_addons::GameEditionAddon;
use edition_paths::GameEditionPaths;

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
                            let names = names.iter()
                                .map(GameEditionAddon::from)
                                .collect();

                            addons.insert(edition, names);
                        }
                    }

                    addons
                })
                .unwrap_or(default.addons)
        })
    }
}
