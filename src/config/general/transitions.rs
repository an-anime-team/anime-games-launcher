use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::LAUNCHER_FOLDER;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transitions {
    pub path: PathBuf
}

impl Default for Transitions {
    #[inline]
    fn default() -> Self {
        Self {
            path: LAUNCHER_FOLDER.join("transitions")
        }
    }
}

impl From<&Json> for Transitions {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            path: value.get("path")
                .and_then(Json::as_str)
                .map(PathBuf::from)
                .unwrap_or(default.path)
        }
    }
}
