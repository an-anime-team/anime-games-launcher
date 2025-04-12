use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Generations {
    pub store: GenerationsStore,

    /// When enabled launcher will use the latest available
    /// generation file and build the new one in background.
    /// Otherwise it will build the new generation and use it.
    ///
    /// Default is true.
    pub lazy_load: bool
}

impl Default for Generations {
    #[inline]
    fn default() -> Self {
        Self {
            store: GenerationsStore::default(),
            lazy_load: true
        }
    }
}

impl AsJson for Generations {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "store": self.store.to_json()?,
            "lazy_load": self.lazy_load
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let default = Self::default();

        Ok(Self {
            store: json.get("store")
                .ok_or_else(|| AsJsonError::FieldNotFound("generations.store"))
                .and_then(GenerationsStore::from_json)
                .unwrap_or(default.store),

            lazy_load: json.get("lazy_load")
                .and_then(Json::as_bool)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("generations.lazy_load"))
                .unwrap_or(default.lazy_load)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenerationsStore {
    pub path: PathBuf
}

impl Default for GenerationsStore {
    fn default() -> Self {
        Self {
            path: DATA_FOLDER.join("generations")
        }
    }
}

impl AsJson for GenerationsStore {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "path": self.path
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            path: json.get("path")
                .ok_or_else(|| AsJsonError::FieldNotFound("generations.store.path"))?
                .as_str()
                .map(PathBuf::from)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("generations.store.path"))?
        })
    }
}
