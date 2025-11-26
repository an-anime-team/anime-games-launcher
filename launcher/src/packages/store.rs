use std::path::{Path, PathBuf};

use serde_json::Value as Json;

use crate::core::prelude::*;
use crate::packages::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Failed to deserialize package manifest: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("Failed to decode package manifest: {0}")]
    AsJson(#[from] AsJsonError)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    folder: PathBuf
}

impl Store {
    #[inline]
    pub fn new(folder: impl Into<PathBuf>) -> Self {
        Self {
            folder: folder.into()
        }
    }

    #[inline]
    /// Get path to the store folder.
    pub fn folder(&self) -> &Path {
        self.folder.as_path()
    }

    /// Build path to the resource in the store.
    pub fn get_path(&self, hash: &Hash, format: &PackageResourceFormat) -> PathBuf {
        let hash = hash.to_base32();

        if format == &PackageResourceFormat::Package {
            self.folder.join(format!("{hash}.src"))
        } else {
            self.folder.join(hash)
        }
    }

    #[inline]
    /// Build path to the temp resource in the store.
    pub fn get_temp_path(&self, hash: &Hash) -> PathBuf {
        self.folder.join(format!("{}.tmp", hash.to_base32()))
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

    #[inline]
    /// Check if an entry with given hash is installed.
    ///
    /// This method will check both resources and packages.
    pub fn has_entry(&self, hash: &Hash) -> bool {
        self.has_resource(hash) || self.has_package(hash)
    }

    /// Try to load package's manifest from the store.
    pub fn get_package(&self, hash: &Hash) -> Result<Option<PackageManifest>, StoreError> {
        let path = self.folder.join(format!("{}.src", hash.to_base32()));

        if !path.exists() {
            return Ok(None);
        }

        let package = std::fs::read(path)?;
        let package = serde_json::from_slice::<Json>(&package)?;

        Ok(Some(PackageManifest::from_json(&package)?))
    }

    /// Validate packages in the lock file.
    ///
    /// This method will scan current store and
    /// validate hashes of the locked resources.
    pub fn validate(&self, lock_file: &LockFileManifest) -> Result<bool, StoreError> {
        // We're doing it this way to validate packages one by one
        // to, potentially, improve performance of the operation.
        for resource in &lock_file.resources {
            let path = self.get_path(&resource.lock.hash, &resource.format);

            if !path.exists() || resource.lock.hash != Hash::for_entry(path)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validate() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-packages-store-test");

        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        let store = Store::new(&path);

        let lock_file = LockFile::with_packages([
            "https://raw.githubusercontent.com/an-anime-team/anime-games-launcher/next/tests/packages/1"
        ]);

        let lock_file = lock_file.build(&store).await
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        let valid = store.validate(&lock_file)
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        assert!(valid);

        Ok(())
    }
}
