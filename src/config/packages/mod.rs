use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::prelude::*;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Packages {
    /// Information about the resources store.
    ///
    /// It is used to download all the packages' resources,
    /// including modules.
    pub resources_store: ResourcesStore,

    /// Information about the modules' store.
    ///
    /// It is used by the modules to store their own,
    /// private information. You can think about it
    /// like it's a browser cookies with longer lifetime.
    pub modules_store: ModulesStore,

    /// Information about the persistent store.
    ///
    /// It is used by the modules to store shared
    /// information. Paths in the persistent store
    /// are indexed using public keys.
    pub persist_store: PersistStore
}

impl AsJson for Packages {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "resources_store": self.resources_store.to_json()?,
            "modules_store": self.modules_store.to_json()?,
            "persist_store": self.persist_store.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            resources_store: json.get("resources_store")
                .map(ResourcesStore::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.resources_store"))??,

            modules_store: json.get("modules_store")
                .map(ModulesStore::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.modules_store"))??,

            persist_store: json.get("persist_store")
                .map(PersistStore::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.persist_store"))??
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourcesStore {
    /// Path to the resources store.
    pub path: PathBuf
}

impl Default for ResourcesStore {
    fn default() -> Self {
        Self {
            path: DATA_FOLDER
                .join("packages")
                .join("resources_store")
        }
    }
}

impl AsJson for ResourcesStore {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "path": self.path
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            path: json.get("path")
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.resources_store.path"))?
                .as_str()
                .map(PathBuf::from)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("packages.resources_store.path"))?
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModulesStore {
    /// Path to the modules' store.
    pub path: PathBuf
}

impl Default for ModulesStore {
    fn default() -> Self {
        Self {
            path: DATA_FOLDER
                .join("packages")
                .join("modules_store")
        }
    }
}

impl AsJson for ModulesStore {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "path": self.path
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            path: json.get("path")
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.modules_store.path"))?
                .as_str()
                .map(PathBuf::from)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("packages.modules_store.path"))?
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersistStore {
    /// Path to the persistent store.
    pub path: PathBuf
}

impl Default for PersistStore {
    fn default() -> Self {
        Self {
            path: DATA_FOLDER
                .join("packages")
                .join("persist_store")
        }
    }
}

impl AsJson for PersistStore {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "path": self.path
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            path: json.get("path")
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.persist_store.path"))?
                .as_str()
                .map(PathBuf::from)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("packages.persist_store.path"))?
        })
    }
}
