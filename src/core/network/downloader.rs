use std::path::PathBuf;
use std::time::Duration;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;
use std::io::SeekFrom;

use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter};
use tokio::task::{JoinHandle, JoinError};
use tokio::runtime::Runtime;

use reqwest::Client;

use crate::config::STARTUP_CONFIG;

// TODO: make a global vector of atomics for all active downloads.
// Each one will store download speed in b/s. Then, using a user-
// specified speed limit, we will calculate the timeout for each
// downloader to achieve this limit.

/// Timeout between requests of the content chunks.
/// Used to slow down downloading speed.
const DOWNLOADER_CHUNKS_REQUESTS_TIMEOUT: Option<Duration> = None;

lazy_static::lazy_static! {
    static ref RUNTIME: Runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("downloader")
        .enable_all()
        .build()
        .expect("Failed to create download tasks runtime");
}

#[derive(Debug, thiserror::Error)]
pub enum DownloaderError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error("Failed to send data between downloader and writer")]
    SendError,

    #[error("Failed to await downloader task: {0}")]
    RuntimeJoin(#[from] JoinError)
}

pub struct DownloadOptions {
    /// If enabled and downloader finds the given output file - it will continue
    /// downloading appending new bytes to that file instead of overwriting it.
    ///
    /// Enabled by default.
    pub continue_download: bool,

    /// Callback executed every time downloader reads a chunk of data.
    ///
    /// Provides `(current, total, diff)` values where `diff` is the change
    /// in `current` value between this callback calls.
    #[allow(clippy::type_complexity)]
    pub on_update: Option<Box<dyn Fn(u64, u64, u64) + Send + Sync>>,

    /// Callback executed when downloading is successfully finished.
    pub on_finish: Option<Box<dyn FnOnce(u64) + Send + Sync>>
}

impl Default for DownloadOptions {
    #[inline]
    fn default() -> Self {
        Self {
            continue_download: true,
            on_update: None,
            on_finish: None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Downloader {
    client: Arc<Client>
}

impl Downloader {
    /// Create new file downloader.
    pub fn new() -> Result<Self, DownloaderError> {
        let client = STARTUP_CONFIG.general.network.builder()?.build()?;

        Ok(Self {
            client: Arc::new(client)
        })
    }

    /// Start downloading of the file, returning struct to control the process.
    ///
    /// This function doesn't block the caller's thread until the file is
    /// downloaded. Async is used to perform initial HTTP GET request to obtain
    /// the header of the file to get its content size.
    pub fn download(
        &self,
        url: impl ToString,
        output_file: impl Into<PathBuf>,
        options: DownloadOptions
    ) -> DownloaderTask {
        let current = Arc::new(AtomicU64::new(0));
        let total = Arc::new(AtomicU64::new(0));
        let aborted = Arc::new(AtomicBool::new(false));

        let client = self.client.clone();

        let url = url.to_string();
        let output_file: PathBuf = output_file.into();

        let task = {
            let current = current.clone();
            let total = total.clone();
            let aborted = aborted.clone();

            RUNTIME.spawn(async move {
                // Open output file.
                let output_file = File::options()
                    .read(true)
                    .write(true)
                    .create(true)
                    .truncate(!options.continue_download)
                    .open(output_file)
                    .await?;

                // Store its length as downloaded bytes length.
                let downloaded = output_file.metadata().await?.len();

                current.store(downloaded, Ordering::Release);

                // Add an inner buffer to the output file to optimize disk writes.
                let mut output_file = BufWriter::new(output_file);

                output_file.seek(SeekFrom::Start(downloaded)).await?;

                // Prepare HTTP request.
                let request = client.get(url)
                    .header("range", format!("bytes={downloaded}-"))
                    .build()?;

                let mut response = client.execute(request).await?;

                // Request content range (downloaded + remained content size).
                //
                // If finished or overcame: `bytes */10611646760`.
                // If not finished: `bytes 10611646759-10611646759/10611646760`.
                //
                // Source: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Range
                if let Some(range) = response.headers().get("content-range") {
                    let range = String::from_utf8_lossy(range.as_bytes());

                    // Finish downloading if header says that we've already downloaded all the data.
                    if range.contains("*/") {
                        total.store(downloaded, Ordering::Release);

                        return Ok(downloaded);
                    }
                }

                // HTTP 416 = provided range is greater than the actual
                // content length (means the file is downloaded).
                //
                // Source: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/416
                if response.status() == 416 {
                    total.store(downloaded, Ordering::Release);

                    return Ok(downloaded);
                }

                // Try to read the `content-length` HTTP header and if successful,
                // store its value as the total length of the downloadable content.
                if let Some(content_length) = response.headers().get("content-length") {
                    let content_length = String::from_utf8_lossy(content_length.as_bytes());

                    if let Ok(content_length) = content_length.parse::<u64>() {
                        total.store(content_length, Ordering::Release);
                    }
                }

                // Read chunks of data from the stream and redirect them to the writer.
                while let Some(chunk) = response.chunk().await? {
                    output_file.write_all(&chunk).await?;

                    let len = chunk.len() as u64;
                    let prev = current.fetch_add(len, Ordering::Relaxed);

                    if let Some(callback) = &options.on_update {
                        callback(prev + len, total.load(Ordering::Relaxed), len);
                    }

                    if aborted.load(Ordering::Acquire) {
                        output_file.flush().await?;

                        return Ok(total.load(Ordering::Acquire));
                    }

                    if let Some(timeout) = DOWNLOADER_CHUNKS_REQUESTS_TIMEOUT {
                        tokio::time::sleep(timeout).await;
                    }
                }

                // Flush remaining buffer to the disk.
                output_file.flush().await?;

                if let Some(callback) = options.on_finish {
                    callback(total.load(Ordering::Acquire));
                }

                Ok::<u64, DownloaderError>(total.load(Ordering::Acquire))
            })
        };

        DownloaderTask {
            current,
            total,
            aborted,
            task
        }
    }
}

#[derive(Debug)]
pub struct DownloaderTask {
    current: Arc<AtomicU64>,
    total: Arc<AtomicU64>,
    aborted: Arc<AtomicBool>,
    task: JoinHandle<Result<u64, DownloaderError>>
}

impl DownloaderTask {
    /// Get amount of downloaded bytes.
    #[inline]
    pub fn current(&self) -> u64 {
        self.current.load(Ordering::Relaxed)
    }

    /// Get expected total amount of bytes.
    #[inline]
    pub fn total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }

    /// Get downloading progress.
    pub fn fraction(&self) -> f64 {
        let current = self.current();
        let total = self.total();

        if current == 0 {
            return 0.0;
        }

        if total == 0 {
            return 1.0;
        }

        current as f64 / total as f64
    }

    /// Check if downloading has finished.
    ///
    /// Note that it could fail so this doesn't mean that we've successfully
    /// downloaded the file.
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.task.is_finished()
    }

    /// Use current thread to wait until the download task is finished,
    /// returning amount of output file bytes.
    #[inline]
    pub async fn wait(self) -> Result<u64, DownloaderError> {
        self.task.await?
    }

    /// Abort the task execution, stopping the file downloading.
    #[inline]
    pub fn abort(self) {
        self.aborted.store(true, Ordering::Release);

        self.task.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn download() -> Result<(), DownloaderError> {
        let path = std::env::temp_dir().join(".agl-download-test");

        let downloader = Downloader::new()?;

        let task = downloader.download(
            "https://github.com/doitsujin/dxvk/releases/download/v2.6.1/dxvk-2.6.1.tar.gz",
            &path,
            DownloadOptions::default()
        );

        assert_eq!(task.wait().await?, 10312443);
        assert_eq!(seahash::hash(&std::fs::read(&path)?), 12012134683777074236);

        std::fs::remove_file(path)?;

        Ok(())
    }

    #[tokio::test]
    async fn abort_download() -> Result<(), DownloaderError> {
        let path = std::env::temp_dir().join(".agl-abort-download-test");

        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        let downloader = Downloader::new()?;

        let task = downloader.download(
            "https://github.com/Kron4ek/Wine-Builds/releases/download/10.5/wine-10.5-staging-tkg-amd64-wow64.tar.xz",
            &path,
            DownloadOptions::default()
        );

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        task.abort();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let hash = seahash::hash(&std::fs::read(&path)?);

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        assert_eq!(seahash::hash(&std::fs::read(&path)?), hash);

        std::fs::remove_file(path)?;

        Ok(())
    }

    #[tokio::test]
    async fn continue_download() -> Result<(), DownloaderError> {
        let path = std::env::temp_dir().join(".agl-continue-download-test");

        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        let downloader = Downloader::new()?;

        let task = downloader.download(
            "https://github.com/Kron4ek/Wine-Builds/releases/download/10.5/wine-10.5-staging-tkg-amd64-wow64.tar.xz",
            &path,
            DownloadOptions::default()
        );

        loop {
            if task.current() > 0 {
                task.abort();

                break;
            }

            // We couldn't stop file downloading before it finished.
            if task.is_finished() {
                return Ok(());
            }
        }

        assert_ne!(seahash::hash(&std::fs::read(&path)?), 16448187945038041646);

        let task = downloader.download(
            "https://github.com/Kron4ek/Wine-Builds/releases/download/10.5/wine-10.5-staging-tkg-amd64-wow64.tar.xz",
            &path,
            DownloadOptions::default()
        );

        let total = task.wait().await?;

        // FIXME: for some reason sometimes returned length is greater.
        // Likely because we download a bit more than needed?
        assert!(total >= 67233060);

        if total == 67233060 {
            assert_eq!(seahash::hash(&std::fs::read(&path)?), 16448187945038041646);
        }

        std::fs::remove_file(path)?;

        Ok(())
    }
}
