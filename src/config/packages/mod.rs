use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::core::prelude::*;
use crate::consts::*;

// Placeholder config. Will be changed in future.

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Packages {
    /// Path where all the packages and their resources
    /// will be stored.
    pub store_path: PathBuf,

    /// Path to the private folders for the modules.
    pub modules_store_path: PathBuf,

    /// Path to the persist folders for the modules.
    pub persist_store_path: PathBuf
}

impl Default for Packages {
    fn default() -> Self {
        Self {
            store_path: DATA_FOLDER
                .join("store")
                .join("resources"),

            modules_store_path: DATA_FOLDER
                .join("store")
                .join("modules_store"),

            persist_store_path: DATA_FOLDER
                .join("store")
                .join("persist_store")
        }
    }
}

impl AsJson for Packages {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "store_path": self.store_path,
            "modules_store_path": self.modules_store_path,
            "persist_store_path": self.persist_store_path
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            store_path: json.get("store_path")
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.store_path"))?
                .as_str()
                .map(PathBuf::from)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("packages.store_path"))?,

            modules_store_path: json.get("modules_store_path")
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.modules_store_path"))?
                .as_str()
                .map(PathBuf::from)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("packages.modules_store_path"))?,

            persist_store_path: json.get("persist_store_path")
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.persist_store_path"))?
                .as_str()
                .map(PathBuf::from)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("packages.persist_store_path"))?
        })
    }
}
