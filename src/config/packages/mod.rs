use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Packages {
    /// List of authority index URLs.
    pub authorities: Vec<String>,

    /// Path to the local resources validator state file.
    pub local_validator: PathBuf,

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
    pub persist_store: PersistStore,

    /// Information about the temporary store.
    ///
    /// It is used by the modules to store temporary
    /// information. Files in this store are eventually
    /// deleted by the garbage collection task.
    pub temp_store: TempStore
}

impl Default for Packages {
    #[inline]
    fn default() -> Self {
        Self {
            authorities: vec![
                String::from("https://raw.githubusercontent.com/an-anime-team/game-integrations/refs/heads/rewrite/packages/authority.json")
            ],

            local_validator: DATA_FOLDER
                .join("packages")
                .join("local_validator.json"),

            resources_store: ResourcesStore::default(),
            modules_store: ModulesStore::default(),
            persist_store: PersistStore::default(),
            temp_store: TempStore::default()
        }
    }
}

impl AsJson for Packages {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "authorities": self.authorities,
            "local_validator": self.local_validator,
            "resources_store": self.resources_store.to_json()?,
            "modules_store": self.modules_store.to_json()?,
            "persist_store": self.persist_store.to_json()?,
            "temp_store": self.temp_store.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        let default = Self::default();

        Ok(Self {
            authorities: json.get("authorities")
                .and_then(Json::as_array)
                .and_then(|authorities| {
                    authorities.iter()
                        .map(|url| url.as_str().map(String::from))
                        .collect::<Option<Vec<_>>>()
                })
                .unwrap_or(default.authorities),

            local_validator: json.get("local_validator")
                .and_then(Json::as_str)
                .map(PathBuf::from)
                .unwrap_or(default.local_validator),

            resources_store: json.get("resources_store")
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.resources_store"))
                .and_then(ResourcesStore::from_json)
                .unwrap_or(default.resources_store),

            modules_store: json.get("modules_store")
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.modules_store"))
                .and_then(ModulesStore::from_json)
                .unwrap_or(default.modules_store),

            persist_store: json.get("persist_store")
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.persist_store"))
                .and_then(PersistStore::from_json)
                .unwrap_or(default.persist_store),

            temp_store: json.get("temp_store")
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.temp_store"))
                .and_then(TempStore::from_json)
                .unwrap_or(default.temp_store)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourcesStore {
    /// Path to the resources store.
    pub path: PathBuf
}

impl Default for ResourcesStore {
    #[inline]
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
    #[inline]
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
    #[inline]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TempStore {
    /// Path to the temporary store.
    pub path: PathBuf
}

impl Default for TempStore {
    #[inline]
    fn default() -> Self {
        Self {
            path: DATA_FOLDER
                .join("packages")
                .join("temp_store")
        }
    }
}

impl AsJson for TempStore {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "path": self.path
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            path: json.get("path")
                .ok_or_else(|| AsJsonError::FieldNotFound("packages.temp_store.path"))?
                .as_str()
                .map(PathBuf::from)
                .ok_or_else(|| AsJsonError::InvalidFieldValue("packages.temp_store.path"))?
        })
    }
}
