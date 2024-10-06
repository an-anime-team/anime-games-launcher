use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::prelude::*;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Generations {
    pub store: GenerationsStore
}

impl AsJson for Generations {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "store": self.store.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            store: json.get("store")
                .map(GenerationsStore::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("generations.store"))??
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
