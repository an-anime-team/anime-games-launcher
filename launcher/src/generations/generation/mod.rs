use std::collections::HashSet;

use serde_json::Value as Json;

use crate::prelude::*;

pub mod manifest;

#[derive(Debug, thiserror::Error)]
pub enum GenerationError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Failed to deserialize manifest: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("Failed to decode manifest: {0}")]
    AsJson(#[from] AsJsonError),

    #[error("Failed to download manifest: {0}")]
    DownloaderError(#[from] DownloaderError),

    #[error("Failed to build lock file for the generation packages: {0}")]
    LockFileError(#[from] LockFileError)
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Generation {
    /// URLs to the games manifests.
    manifests: HashSet<String>
}

impl Generation {
    #[inline]
    /// Create new generation file from the provided links.
    ///
    /// Use `Generation::default` if you want an empty generation.
    pub fn new<T: ToString>(games: impl IntoIterator<Item = T>) -> Self {
        Self {
            manifests: games.into_iter()
                .map(|url| url.to_string())
                .collect()
        }
    }

    // TODO: fail-tolerant building + proper async use.
    // TODO: fast rebuild method to insert one new entry and don't update other ones.

    /// Build new generation from provided URLs.
    ///
    /// Note: This is a heavy function which executes
    /// lock file building internally. It's expected
    /// to run it in background.
    pub async fn build(&self, packages_store: &PackagesStore, generations_store: &GenerationsStore) -> Result<GenerationManifest, GenerationError> {
        // Prepare set of packages to be locked.
        let mut packages = HashSet::with_capacity(self.manifests.len());

        // Start downloading all added games' manifests.
        let mut download_tasks = Vec::with_capacity(self.manifests.len());

        tracing::trace!("Fetching games manifests");

        let downloader = Downloader::new()?;

        for url in &self.manifests {
            let temp_hash = Hash::rand();
            let temp_path = generations_store.get_temp_path(&temp_hash);

            tracing::trace!(?url, ?temp_path, "Fetching game manifest");

            let task = downloader.download(url, &temp_path, DownloadOptions::default());

            download_tasks.push((url.clone(), temp_path, task));
        }

        // Await games' manifests and store game packages URLs.
        let mut games = Vec::with_capacity(download_tasks.len());

        for (url, temp_path, task) in download_tasks.drain(..) {
            // Await manifest download task.
            task.wait().await?;

            tracing::trace!(?url, ?temp_path, "Processing game manifest");

            // Parse the manifest file.
            let manifest = std::fs::read(&temp_path)?;
            let manifest = serde_json::from_slice::<Json>(&manifest)?;
            let manifest = GameManifest::from_json(&manifest)?;

            // Delete it.
            std::fs::remove_file(temp_path)?;

            // Store the manifest and the game package's URL
            // to build a lock file.
            packages.insert(manifest.package.url.clone());

            games.push(GenerationGameLock {
                url,
                manifest
            });
        }

        tracing::trace!("Building lock file for the generation");

        // Build the lock file for the game packages.
        let lock_file = LockFile::with_packages(packages)
            .build(packages_store)
            .await?;

        Ok(GenerationManifest::compose(games, lock_file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn build() -> Result<(), GenerationError> {
        let path = std::env::temp_dir().join(".agl-generations-test");

        if path.exists() {
            std::fs::remove_dir_all(&path)?;
        }

        std::fs::create_dir_all(&path)?;

        // Use the same folder for both packages and generations.
        let packages_store = PackagesStore::new(&path);
        let generations_store = GenerationsStore::new(&path);

        let generation = Generation::new([
            "https://raw.githubusercontent.com/an-anime-team/anime-games-launcher/next/tests/games/1.json"
        ]);

        let generation = generation.build(&packages_store, &generations_store).await?;

        assert_eq!(generation.games.len(), 1);
        assert_eq!(generation.lock_file.root.iter().copied().sum::<u32>(), 0);
        assert_eq!(generation.lock_file.resources.len(), 8);
        assert_eq!(Hash::for_entry(path)?, Hash(5516354445018355056));

        Ok(())
    }
}
