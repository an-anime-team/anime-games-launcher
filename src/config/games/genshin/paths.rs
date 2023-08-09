use std::path::{Path, PathBuf};

use serde::{Serialize, Deserialize};

use serde_json::Value as Json;

use anime_game_core::game::genshin::Edition;

use crate::LAUNCHER_FOLDER;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paths {
    pub global: PathBuf,
    pub china: PathBuf
}

impl Default for Paths {
    #[inline]
    fn default() -> Self {
        Self {
            global: LAUNCHER_FOLDER.join("games/genshin-impact/global"),
            china: LAUNCHER_FOLDER.join("games/genshin-impact/china")
        }
    }
}

impl Paths {
    #[inline]
    pub fn for_edition(&self, edition: Edition) -> &Path {
        match edition {
            Edition::Global => self.global.as_path(),
            Edition::China  => self.china.as_path()
        }
    }
}

impl From<&Json> for Paths {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            global: value.get("global")
                .and_then(Json::as_str)
                .map(PathBuf::from)
                .unwrap_or(default.global),

            china: value.get("china")
                .and_then(Json::as_str)
                .map(PathBuf::from)
                .unwrap_or(default.china),
        }
    }
}
