use std::path::{Path, PathBuf};
use std::collections::HashMap;

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

    /// Make a wrapper struct around existing resource
    /// with given hash using the lock file.
    ///
    /// Return none if there's no resource with this hash
    /// in the store.
    pub fn load(&self, lock_file: &LockFileManifest, hash: &Hash) -> Result<Option<Resource>, StoreError> {
        // Find the resource info in the lock file.
        let resource = lock_file.resources.iter()
            .find(|resource| resource.lock.hash == *hash);

        let Some(resource) = resource else {
            return Ok(None);
        };

        let hash = resource.lock.hash;

        // If it's not a package - find its type
        // and return the wrapper.
        if resource.format != PackageResourceFormat::Package {
            let path = self.folder.join(hash.to_base32());

            if path.is_file() {
                Ok(Some(Resource::File {
                    hash,
                    path
                }))
            } else if path.is_dir() {
                Ok(Some(Resource::Folder {
                    hash,
                    path
                }))
            } else {
                Ok(None)
            }
        }

        // Otherwise - process all the inputs
        // and outputs of the package and return it.
        else {
            let Some(manifest) = self.get_package(&hash)? else {
                return Ok(None);
            };

            let path = self.folder.join(format!("{}.src", resource.lock.hash.to_base32()));

            Ok(Some(Resource::Package {
                hash,
                path,

                metadata: manifest.package,

                inputs: resource.inputs.clone()
                    .unwrap_or_default(),

                outputs: resource.outputs.clone()
                    .unwrap_or_default()
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn read_packages() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-read-packages-test");

        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        let store = Store::new(&path);

        let lock_file = LockFile::with_packages(store.clone(), [
            "https://raw.githubusercontent.com/an-anime-team/anime-games-launcher/next/tests/packages/1"
        ]);

        let lock_file = lock_file.build().await
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        assert_eq!(lock_file.root, &[Hash(14823907562133104457)]);
        assert_eq!(lock_file.resources.len(), 6);
        assert_eq!(Hash::for_entry(path)?, Hash(6776203643455837073));

        let root_package = store.load(&lock_file, &Hash(14823907562133104457))
            .map_err(|err| anyhow::anyhow!(err.to_string()))?
            .ok_or_else(|| anyhow::anyhow!("No root package read"))?;

        assert_eq!(root_package.get_hash(), &Hash(14823907562133104457));

        let Some(inputs) = root_package.get_inputs() else {
            anyhow::bail!("No inputs in the root package");
        };

        let Some(outputs) = root_package.get_outputs() else {
            anyhow::bail!("No outputs in the root package");
        };

        assert_eq!(inputs["self-reference"], Hash(14823907562133104457));
        assert_eq!(inputs["another-package"], Hash(16664589923667942635));
        assert_eq!(outputs["self-reference"], Hash(3622836511576447158));

        Ok(())
    }
}
