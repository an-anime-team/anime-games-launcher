use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

pub mod edition_addons;
pub mod edition_paths;

pub mod prelude {
    pub use super::edition_addons::GameEditionAddon;
    pub use super::edition_paths::GameEditionPaths;
    pub use super::GameSettings;
}

use prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSettings {
    pub paths: HashMap<String, GameEditionPaths>,
    pub addons: HashMap<String, Vec<GameEditionAddon>>
}

impl GameSettings {
    pub fn default(game_name: impl AsRef<str>, edition_names: impl IntoIterator<Item = impl AsRef<str>> + Clone) -> anyhow::Result<Self> {
        Ok(Self {
            paths: edition_names
                .clone()
                .into_iter()
                .filter_map(|edition| {
                    match GameEditionPaths::default(game_name.as_ref(), edition.as_ref()) {
                        Ok(paths) => Some((edition.as_ref().to_string(), paths)),
                        Err(_) => None
                    }
                }).collect(),

            addons: edition_names
                .into_iter()
                .map(|edition| (edition.as_ref().to_string(), vec![]))
                .collect::<HashMap<_, _>>(),
        })
    }

    pub fn from_json(game_name: impl AsRef<str>, edition_names: impl IntoIterator<Item = impl AsRef<str>> + Clone, value: &Json) -> anyhow::Result<Self> {
        let mut default = Self::default(game_name.as_ref(), edition_names)?;

        if let Some(values) = value.get("paths").and_then(Json::as_object) {
            for (edition, edition_paths) in values.clone() {
                if let Ok(value) = GameEditionPaths::from_json(game_name.as_ref(), &edition, &edition_paths) {
                    default.paths.insert(edition, value);
                }
            }
        }

        if let Some(values) = value.get("addons").and_then(Json::as_object) {
            for (edition, names) in values.clone() {
                if let Some(names) = names.as_array() {
                    let names = names.iter()
                        .map(GameEditionAddon::from)
                        .collect();

                    default.addons.insert(edition, names);
                }
            }
        }

        Ok(Self {
            paths: default.paths,
            addons: default.addons
        })
    }
}
