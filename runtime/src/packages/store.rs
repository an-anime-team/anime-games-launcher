use std::path::{Path, PathBuf};

use toml::Table as TomlTable;

use crate::hash::Hash;

use super::manifest::{PackageManifest, PackageManifestError};
use super::lock_file::LockFile;

#[derive(Debug, thiserror::Error)]
pub enum ResourceStoreError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Serialize(#[from] toml::de::Error),

    #[error(transparent)]
    PackageManifestError(#[from] PackageManifestError)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceStore {
    folder: PathBuf
}

impl ResourceStore {
    #[inline]
    pub fn new(folder: impl Into<PathBuf>) -> Self {
        Self {
            folder: folder.into()
        }
    }

    /// Get path to the store folder.
    #[inline]
    pub fn folder(&self) -> &Path {
        self.folder.as_path()
    }

    /// Build path to the resource in the store.
    #[inline]
    pub fn get_path(&self, hash: &Hash) -> PathBuf {
        self.folder.join(hash.to_base32())
    }

    /// Build path to the temp resource in the store.
    #[inline]
    pub fn get_temp_path(&self, hash: &Hash) -> PathBuf {
        self.folder.join(format!("{}.tmp", hash.to_base32()))
    }

    /// Check if a resource with given hash is installed.
    #[inline]
    pub fn has_resource(&self, hash: &Hash) -> bool {
        self.folder.join(hash.to_base32()).exists()
    }

    /// Try to load package's manifest from the store.
    pub fn get_package(&self, hash: &Hash) -> Result<Option<PackageManifest>, ResourceStoreError> {
        let path = self.folder.join(hash.to_base32());

        if !path.exists() {
            return Ok(None);
        }

        let package = std::fs::read_to_string(path)?;
        let package = toml::from_str::<TomlTable>(&package)?;

        Ok(Some(PackageManifest::try_from(&package)?))
    }

    /// Validate packages in the lock file.
    ///
    /// This method will scan current store and validate hashes of the locked
    /// resources.
    pub fn validate(&self, lock_file: &LockFile) -> Result<bool, ResourceStoreError> {
        // We're doing it this way to validate packages one by one
        // to, potentially, improve performance of the operation.
        for resource in &lock_file.resources {
            let path = self.get_path(&resource.lock.hash);

            if !path.exists() || resource.lock.hash != Hash::for_entry(path)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
