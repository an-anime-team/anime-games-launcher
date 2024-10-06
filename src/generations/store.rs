use std::path::{Path, PathBuf};

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
        let generations = serde_json::from_slice::<Json>(&generations)?;

        let mut generations = GenerationsManifest::from_json(&generations)?.generations
            .into_iter()
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

    /// Insert generation to the store.
    pub fn insert(&self, generation: &GenerationManifest) -> Result<(), StoreError> {
        let hash = generation.partial_hash();

        let manifest_path = self.folder.join("generations.json");
        let generation_path = self.folder.join(hash.to_base32());

        let mut generations = if !manifest_path.exists() {
            GenerationsManifest::default()
        } else {
            let generations = std::fs::read(&manifest_path)?;
            let generations = serde_json::from_slice::<Json>(&generations)?;

            GenerationsManifest::from_json(&generations)?
        };

        generations.generations.insert(hash, generation.generated_at);

        std::fs::write(manifest_path, serde_json::to_vec_pretty(&generations.to_json()?)?)?;
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
        let generations = serde_json::from_slice::<Json>(&generations)?;

        let mut generations = GenerationsManifest::from_json(&generations)?;

        generations.generations.remove(generation);

        std::fs::write(manifest_path, serde_json::to_vec_pretty(&generations.to_json()?)?)?;

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

    #[tokio::test]
    async fn store() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-generations-store-test");

        if path.exists() {
            std::fs::remove_dir_all(&path)?;
        }

        std::fs::create_dir_all(&path)?;

        // Use the same folder for both packages and generations.
        let packages_store = PackagesStore::new(&path);
        let generations_store = GenerationsStore::new(&path);

        assert!(generations_store.list().map_err(|err| anyhow::anyhow!(err.to_string()))?.is_none());
        assert!(generations_store.latest().map_err(|err| anyhow::anyhow!(err.to_string()))?.is_none());
        assert!(!generations_store.has_generation(&Hash(535491346813091909)));

        let generation = Generation::with_games([
            "https://raw.githubusercontent.com/an-anime-team/anime-games-launcher/next/tests/games/1.json"
        ]);

        let generation = generation.build(&packages_store, &generations_store).await
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        assert_eq!(generation.games.len(), 1);
        assert_eq!(&generation.lock_file.root, &[0]);
        assert_eq!(generation.lock_file.resources.len(), 8);
        assert_eq!(Hash::for_entry(path)?, Hash(9585216612201553270));

        generations_store.insert(&generation)
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        assert!(generations_store.list().map_err(|err| anyhow::anyhow!(err.to_string()))?.is_some());
        assert!(generations_store.latest().map_err(|err| anyhow::anyhow!(err.to_string()))?.is_some());
        assert!(generations_store.has_generation(&Hash(18086654289878451496)));

        let generation = generations_store.load(&Hash(18086654289878451496))
            .map_err(|err| anyhow::anyhow!(err.to_string()))?
            .ok_or_else(|| anyhow::anyhow!("Generation expected, got none"))?;

        assert_eq!(generation.games.len(), 1);
        assert_eq!(&generation.lock_file.root, &[0]);
        assert_eq!(generation.lock_file.resources.len(), 8);

        generations_store.remove(&Hash(18086654289878451496))
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        assert!(generations_store.latest().map_err(|err| anyhow::anyhow!(err.to_string()))?.is_none());
        assert!(!generations_store.has_generation(&Hash(535491346813091909)));

        // TODO: would be good to insert couple more generations to verify
        // their ordering and deduplication mechanism.

        Ok(())
    }
}
