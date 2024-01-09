use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::LAUNCHER_FOLDER;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Integrations {
    pub sources: Vec<String>,
    pub path: PathBuf
}

impl Default for Integrations {
    #[inline]
    fn default() -> Self {
        Self {
            sources: vec![
                String::from("https://raw.githubusercontent.com/an-anime-team/game-integrations/main")
            ],

            path: LAUNCHER_FOLDER.join("integrations")
        }
    }
}

impl From<&Json> for Integrations {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            sources: value.get("sources")
                .and_then(Json::as_array)
                .map(|sources| sources.iter()
                    .filter_map(|source| source.as_str()
                    .map(String::from))
                    .collect()
                )
                .unwrap_or(default.sources),

            path: value.get("path")
                .and_then(Json::as_str)
                .map(PathBuf::from)
                .unwrap_or(default.path)
        }
    }
}
