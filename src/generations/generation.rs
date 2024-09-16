use std::collections::HashSet;

use serde_json::Value as Json;

use crate::core::prelude::*;
use crate::packages::prelude::*;
use crate::games::prelude::*;

use super::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum GenerationError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Failed to deserialize game manifest: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("Failed to decode game manifest: {0}")]
    AsJson(#[from] AsJsonError),

    #[error("Failed to download game manifest: {0}")]
    DownloaderError(#[from] DownloaderError),

    #[error("Failed to build lock file for game packages: {0}")]
    LockFileError(#[from] LockFileError)
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Generation {
    /// URLs to the games manifests for the generation file.
    manifest_urls: HashSet<String>
}

impl Generation {
    #[inline]
    /// Create new empty generations file.
    pub fn new() -> Self {
        Self {
            manifest_urls: HashSet::new(),
        }
    }

    #[inline]
    /// Create new generations file with given games manifests URLs.
    pub fn with_games<T: Into<String>>(manifests: impl IntoIterator<Item = T>) -> Self {
        Self {
            manifest_urls: HashSet::from_iter(manifests.into_iter().map(T::into)),
        }
    }

    #[inline]
    /// Add game manifest URL.
    pub fn add_game(&mut self, url: impl ToString) -> &mut Self {
        self.manifest_urls.insert(url.to_string());

        self
    }

    /// Build new generation from provided URLs
    /// to the games' manifests.
    ///
    /// Note: This is a heavy function which executes
    /// lock file building internally. You should
    /// run it in background.
    pub async fn build<T: ToString>(&self, packages_store: &PackagesStore, generations_store: &GenerationsStore) -> Result<GenerationManifest, GenerationError> {
        let mut games_contexts = Vec::with_capacity(self.manifest_urls.len());

        // Start downloading all added games' manifests.
        for url in self.manifest_urls.clone() {
            let temp_hash = Hash::rand();
            let temp_path = generations_store.get_temp_path(&temp_hash);

            let context = Downloader::new(&url)?
                .with_continue_downloading(false)
                .with_output_file(&temp_path)
                .download(|_, _, _| {})
                .await?;

            games_contexts.push((url, temp_path, context));
        }

        // Await game manifests and store game packages URLs.
        let mut packages = HashSet::with_capacity(games_contexts.len());
        let mut games = Vec::with_capacity(games_contexts.len());

        for (url, temp_path, context) in games_contexts.drain(..) {
            // Await manifest download finish.
            context.wait()?;

            // Parse the manifest file.
            let manifest = std::fs::read(&temp_path)?;
            let manifest = serde_json::from_slice::<Json>(&manifest)?;
            let manifest = GameManifest::from_json(&manifest)?;

            // Delete it.
            std::fs::remove_file(temp_path)?;

            // Store the manifest and the game's package URL
            // to build a lock file.
            packages.insert(manifest.package.url.clone());

            games.push(GenerationGameLock {
                url,
                manifest
            });
        }

        // Build the lock file for the game packages.
        let lock_file = LockFile::with_packages(packages)
            .build(packages_store)
            .await?;

        Ok(GenerationManifest::compose(games, lock_file))
    }
}
