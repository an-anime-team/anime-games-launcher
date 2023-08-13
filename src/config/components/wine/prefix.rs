use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use serde_json::Value as Json;

use crate::LAUNCHER_FOLDER;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prefix {
    pub path: PathBuf,
    pub install_corefonts: bool
}

impl Default for Prefix {
    #[inline]
    fn default() -> Self {
        Self {
            path: LAUNCHER_FOLDER
                .join("components")
                .join("prefix"),

            install_corefonts: true
        }
    }
}

impl From<&Json> for Prefix {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            path: value.get("path")
                .and_then(Json::as_str)
                .map(PathBuf::from)
                .unwrap_or(default.path),

            install_corefonts: value.get("install_corefonts")
                .and_then(Json::as_bool)
                .unwrap_or(default.install_corefonts)
        }
    }
}
