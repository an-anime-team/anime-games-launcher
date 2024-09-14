use std::path::{Path, PathBuf};

use serde_json::Value as Json;

use crate::core::prelude::*;
use crate::packages::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Serialize(#[from] serde_json::Error),

    #[error(transparent)]
    AsJson(#[from] AsJsonError)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    folder: PathBuf
}

impl Store {
    #[inline]
    /// Get path to the store folder.
    pub fn folder(&self) -> &Path {
        self.folder.as_path()
    }

    #[inline]
    /// Check if a resource with given hash is installed.
    pub fn has_resource(&self, hash: &Hash) -> bool {
        self.folder.join(hash.to_base32()).exists()
    }

    #[inline]
    /// Check if a package with given hash is installed.
    pub fn has_package(&self, hash: &Hash) -> bool {
        self.folder.join(format!("{}.src", hash.to_base32())).exists()
    }

    /// Try to load package from the store.
    pub fn get_package(&self, hash: &Hash) -> Result<Option<PackageManifest>, StoreError> {
        let path = self.folder.join(format!("{}.src", hash.to_base32()));

        if !path.exists() {
            return Ok(None);
        }

        let package = std::fs::read(path)?;
        let package = serde_json::from_slice::<Json>(&package)?;

        Ok(Some(PackageManifest::from_json(&package)?))
    }
}
