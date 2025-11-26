use std::path::PathBuf;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::io::SeekFrom;

use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter};
use tokio::task::{JoinError, JoinHandle};
use reqwest::Client;

// TODO: make a global vector of atomics for all active downloads.
// Each one will store download speed in b/s. Then, using a user-
// specified speed limit, we will calculate the timeout for each
// downloader to achieve this limit.

/// Timeout between requests of the content chunks.
/// Used to slow down downloading speed.
const DOWNLOADER_CHUNKS_REQUESTS_TIMEOUT: Option<Duration> = None;

lazy_static::lazy_static! {
    static ref CLIENT: Client = Client::new();
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
pub struct Downloader(Client);

impl Default for Downloader {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl Downloader {
    /// Create new file downloader using shared reqwest client.
    #[inline]
    pub fn new() -> Self {
        #[cfg(feature = "tracing")]
        tracing::trace!("create default downloader");

        Self(CLIENT.clone())
    }

    /// Create new file downloader from the given reqwest client.
    #[inline(always)]
    pub const fn from_client(client: Client) -> Self {
        Self(client)
    }

    /// Start downloading of the file using default options.
    #[inline]
    pub fn download(&self, url: impl ToString, output_file: impl Into<PathBuf>) -> DownloaderTask {
        self.download_with_options(url, output_file, DownloadOptions::default())
    }

    /// Start downloading of the file, returning struct to control the process.
    ///
    /// This function doesn't block the caller's thread until the file is
    /// downloaded. Async is used to perform initial HTTP GET request to obtain
    /// the header of the file to get its content size.
    pub fn download_with_options(
        &self,
        url: impl ToString,
        output_file: impl Into<PathBuf>,
        options: DownloadOptions
    ) -> DownloaderTask {
        let url = url.to_string();
        let output_file: PathBuf = output_file.into();

        #[cfg(feature = "tracing")]
        tracing::trace!(
            ?url,
            ?output_file,
            continue_download = options.continue_download,
            "start downloading"
        );

        let current = Arc::new(AtomicU64::new(0));
        let total = Arc::new(AtomicU64::new(0));
        let aborted = Arc::new(AtomicBool::new(false));

        let client = self.0.clone();

        let task = {
            let current = current.clone();
            let total = total.clone();
            let aborted = aborted.clone();

            crate::tasks::spawn(async move {
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
                let request = client
                    .get(url)
                    .header("range", format!("bytes={downloaded}-"))
                    .build()?;

                let mut response = client.execute(request).await?;

                // HTTP 416 = provided range is greater than the actual
                // content length (means the file is downloaded).
                //
                // Source: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/416
                if response.status() == 416 {
                    total.store(downloaded, Ordering::Release);

                    return Ok(downloaded);
                }

                // Try to read the `Content-Length` HTTP header and if successful,
                // store its value as the partial length of downloadable content.
                if let Some(content_length) = response.headers().get("Content-Length") {
                    let content_length = String::from_utf8_lossy(content_length.as_bytes());

                    if let Ok(content_length) = content_length.parse::<u64>() {
                        // If we already downloaded part of the content -
                        // `Content-Length` will contain a length of the
                        // remaining content.
                        total.store(downloaded + content_length, Ordering::Release);
                    }
                }

                // Request content range (downloaded + remained content size).
                //
                // If finished or overcame: `bytes */10611646760`.
                // If not finished: `bytes 10611646759-10611646759/10611646760`.
                //
                // Content-Range: <unit> <range>/<size>
                // Content-Range: <unit> <range>/*
                // Content-Range: <unit> */<size>
                //
                // Source: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Range
                if let Some(range) = response.headers().get("Content-Range") {
                    let range = String::from_utf8_lossy(range.as_bytes());

                    if let Some(range) = range.strip_prefix("bytes ") {
                        if let Some((range, size)) = range.split_once('/') {
                            // Downloading finished.
                            if range == "*" {
                                total.store(downloaded, Ordering::Release);

                                return Ok(downloaded);
                            }

                            if let Ok(size) = size.parse::<u64>() {
                                total.store(size, Ordering::Release);
                            }
                        }
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
        let path = std::env::temp_dir().join(".wineyard-core-download-test");

        let downloader = Downloader::default();

        let task = downloader.download(
            "https://github.com/doitsujin/dxvk/releases/download/v2.6.1/dxvk-2.6.1.tar.gz",
            &path
        );

        assert_eq!(task.wait().await?, 10312443);

        std::fs::remove_file(path)?;

        Ok(())
    }

    #[tokio::test]
    async fn abort_download() -> Result<(), DownloaderError> {
        let path = std::env::temp_dir().join(".wineyard-core-abort-download-test");

        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        let downloader = Downloader::default();

        let task = downloader.download(
            "https://github.com/Kron4ek/Wine-Builds/releases/download/10.5/wine-10.5-staging-tkg-amd64-wow64.tar.xz",
            &path
        );

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        task.abort();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let len = std::fs::metadata(&path)?.len();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        assert_eq!(std::fs::metadata(&path)?.len(), len);

        std::fs::remove_file(path)?;

        Ok(())
    }

    #[tokio::test]
    async fn continue_download() -> Result<(), DownloaderError> {
        let path = std::env::temp_dir().join(".wineyard-core-continue-download-test");

        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        let downloader = Downloader::default();

        let task = downloader.download(
            "https://github.com/Kron4ek/Wine-Builds/releases/download/10.5/wine-10.5-staging-tkg-amd64-wow64.tar.xz",
            &path
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

        assert_ne!(std::fs::metadata(&path)?.len(), 10312443);

        let task = downloader.download(
            "https://github.com/Kron4ek/Wine-Builds/releases/download/10.5/wine-10.5-staging-tkg-amd64-wow64.tar.xz",
            &path
        );

        let total = task.wait().await?;

        // FIXME: for some reason sometimes returned length is greater.
        // Likely because we download a bit more than needed?
        assert!(total >= 67233060);

        std::fs::remove_file(path)?;

        Ok(())
    }
}
