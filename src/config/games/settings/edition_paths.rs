use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::LAUNCHER_FOLDER;

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
