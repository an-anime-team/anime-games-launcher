use serde::{Serialize, Deserialize};

use serde_json::Value as Json;

use anime_game_core::game::genshin::Edition;

use crate::LAUNCHER_FOLDER;

use crate::config::driver::Driver;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paths {
    pub global: Driver,
    pub china: Driver
}

impl Default for Paths {
    #[inline]
    fn default() -> Self {
        let game_folder = LAUNCHER_FOLDER
            .join("games")
            .join(concat!("Gen", "shin Im", "pact"));

        Self {
            global: Driver::PhysicalFsDriver {
                base_folder: game_folder.join("global")
            },

            china: Driver::PhysicalFsDriver {
                base_folder: game_folder.join("china")
            }
        }
    }
}

impl Paths {
    #[inline]
    pub fn for_edition(&self, edition: Edition) -> &Driver {
        match edition {
            Edition::Global => &self.global,
            Edition::China  => &self.china
        }
    }
}

impl From<&Json> for Paths {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            global: value.get("global")
                .and_then(|value| serde_json::from_value(value.clone()).ok())
                .unwrap_or(default.global),

            china: value.get("china")
                .and_then(|value| serde_json::from_value(value.clone()).ok())
                .unwrap_or(default.china),
        }
    }
}
