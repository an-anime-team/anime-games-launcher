use std::path::PathBuf;
use std::time::Duration;

use tokio::sync::oneshot::Sender;

use crate::prelude::*;

pub struct FileCache {
    path: PathBuf,
    cache_time: Duration
}

impl Default for FileCache {
    #[inline]
    fn default() -> Self {
        Self::new(CACHE_FOLDER.as_path())
    }
}

impl FileCache {
    #[inline]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            cache_time: Duration::from_secs(8 * 3600)
        }
    }

    #[inline]
    pub fn with_cache_time(mut self, cache_time: Duration) -> Self {
        self.cache_time = cache_time;

        self
    }

    /// Download file from the given URL and store it to the cache folder.
    /// Return path to the previous cached version of this file.
    /// When file is downloaded - a given callback will be called.
    ///
    /// ```
    /// let cache = FileCache::default();
    ///
    /// let prev = cache.swap("https://static.wikia.nocookie.net/amogus/images/3/31/Red.png", |new| {
    ///     // Use the new cache of the file.
    ///     dbg!(new);
    /// });
    ///
    /// // Use the previous cached file if avaialble.
    /// dbg!(prev);
    /// ```
    pub fn swap(&self, url: impl ToString, callback: Sender<PathBuf>) -> Option<PathBuf> {
        let url = url.to_string();

        let key = Hash::for_slice(url.as_bytes());

        let cache_path = self.path.join(key.to_base32());
        let swap_path = self.path.join(Hash::rand().to_base32());

        {
            let cache_path = cache_path.clone();
            let cache_time = self.cache_time;

            tokio::spawn(async move {
                tracing::trace!(?cache_path, ?swap_path, ?url, "Starting cache file renewing");

                if let Ok(metadata) = cache_path.metadata() {
                    if let Ok(created_at) = metadata.created() {
                        if let Ok(lifetime) = created_at.elapsed() {
                            if lifetime < cache_time {
                                tracing::trace!(?cache_path, ?swap_path, ?url, "File is cached recently, no renewing needed");

                                return;
                            }
                        }
                    }
                }

                let downloader = match Downloader::new(&url) {
                    Ok(downloader) => downloader
                        .with_continue_downloading(false)
                        .with_output_file(&swap_path),

                    Err(err) => {
                        tracing::error!(?url, ?err, "Failed to open cache file downloader");

                        return;
                    }
                };

                let context = match downloader.download(|_, _, _| {}).await {
                    Ok(context) => context,
                    Err(err) => {
                        tracing::error!(?err, "Failed to start renewing cache file");

                        let _ = tokio::fs::remove_file(&swap_path).await;

                        return;
                    }
                };

                if let Err(err) = context.wait() {
                    tracing::error!(?err, "Failed to renew cache file");

                    let _ = tokio::fs::remove_file(&swap_path).await;

                    return;
                }

                if let Err(err) = tokio::fs::rename(&swap_path, &cache_path).await {
                    tracing::error!(?err, "Failed to replace old cache file");

                    let _ = tokio::fs::remove_file(&swap_path).await;

                    return;
                }

                tracing::trace!(?cache_path, ?url, "Finished cache file renewing");

                let _ = callback.send(cache_path);
            });
        }

        cache_path.exists().then_some(cache_path)
    }
}
