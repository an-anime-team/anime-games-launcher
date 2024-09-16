use std::path::{Path, PathBuf};
use std::collections::HashMap;

use serde_json::Value as Json;

use crate::core::prelude::*;
use crate::packages::prelude::*;

use super::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Failed to deserialize generation file: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("Failed to decode generation manifest: {0}")]
    AsJson(#[from] AsJsonError)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Store {
    folder: PathBuf
}

impl Store {
    #[inline]
    /// Create new empty generations store.
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

    #[inline]
    /// Build path to the generation in the store.
    pub fn get_path(&self, generation: &Hash) -> PathBuf {
        self.folder.join(generation.to_base32())
    }

    #[inline]
    /// Build path to the temp generation in the store.
    pub fn get_temp_path(&self, generation: &Hash) -> PathBuf {
        self.folder.join(format!("{}.tmp", generation.to_base32()))
    }

    #[inline]
    /// Check if generation exists in the store.
    pub fn has_generation(&self, generation: &Hash) -> bool {
        self.get_path(generation).exists()
    }

    /// Return list of all available generations
    /// in ascending order (later is newer).
    pub fn list(&self) -> Result<Option<Vec<Hash>>, StoreError> {
        let path = self.folder.join("generations.json");

        if !path.exists() {
            return Ok(None);
        }

        let generations = std::fs::read(path)?;
        let generations = serde_json::from_slice::<HashMap<Hash, u64>>(&generations)?;

        let mut generations = generations.into_iter()
            .collect::<Vec<_>>();

        generations.sort_by_key(|(_, time)| *time);

        let generations = generations.into_iter()
            .map(|(hash, _)| hash)
            .collect();

        Ok(Some(generations))
    }

    /// Get latest available generation.
    pub fn latest(&self) -> Result<Option<Hash>, StoreError> {
        match self.list()? {
            Some(generations) => Ok(generations.last().copied()),
            None => Ok(None)
        }
    }

    pub fn insert(&self, generation: GenerationManifest) -> Result<(), StoreError> {
        let hash = generation.hash();

        let manifest_path = self.folder.join("generations.json");
        let generation_path = self.folder.join(hash.to_base32());

        if !manifest_path.exists() {
            std::fs::write(&manifest_path, "{}")?;
        }

        let generations = std::fs::read(&manifest_path)?;
        let mut generations = serde_json::from_slice::<HashMap<Hash, u64>>(&generations)?;

        generations.insert(hash, generation.generated_at);

        std::fs::write(manifest_path, serde_json::to_vec_pretty(&generations)?)?;
        std::fs::write(generation_path, serde_json::to_vec_pretty(&generation.to_json()?)?)?;

        Ok(())
    }

    /// Remove generation from the store.
    pub fn remove(&self, generation: &Hash) -> Result<(), StoreError> {
        let manifest_path = self.folder.join("generations.json");
        let generation_path = self.folder.join(generation.to_base32());

        if !manifest_path.exists() {
            return Ok(());
        }

        let generations = std::fs::read(&manifest_path)?;
        let mut generations = serde_json::from_slice::<HashMap<Hash, u64>>(&generations)?;

        generations.remove(generation);

        std::fs::write(manifest_path, serde_json::to_vec_pretty(&generations)?)?;

        if generation_path.exists() {
            std::fs::remove_file(generation_path)?;
        }

        Ok(())
    }

    /// Try to load generation from the store.
    pub fn load(&self, generation: &Hash) -> Result<Option<GenerationManifest>, StoreError> {
        let path = self.folder.join(generation.to_base32());

        if !path.exists() {
            return Ok(None);
        }

        let generation = std::fs::read(path)?;
        let generation = serde_json::from_slice::<Json>(&generation)?;

        Ok(Some(GenerationManifest::from_json(&generation)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() -> Result<(), StoreError> {
        let path = std::env::temp_dir().join(".agl-generations-test");

        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        let generations = Store::new(path);

        assert_eq!(generations.list()?, None);
        assert_eq!(generations.latest()?, None);
        assert!(!generations.has_generation(&Hash::rand()));

        Ok(())
    }
}
