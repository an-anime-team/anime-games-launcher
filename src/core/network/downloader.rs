use std::path::{Path, PathBuf};
use std::time::Duration;
use std::thread::JoinHandle;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::io::SeekFrom;

use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, BufWriter};
use tokio::task::JoinError;
use tokio::runtime::Runtime;

use reqwest::Client;

use crate::consts::DATA_FOLDER;
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
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Failed to send data between downloader and writer")]
    SendError,

    #[error("Failed to join async runtime: {0}")]
    RuntimeJoin(#[from] JoinError),

    #[error("Failed to join downloader thread: {0:?}")]
    ThreadJoin(Box<dyn std::any::Any + Send + 'static>)
}

#[derive(Debug)]
pub struct Downloader {
    client: Client,

    input_url: String,
    output_file: PathBuf,
    continue_downloading: bool
}

impl Downloader {
    /// Create new file downloader.
    pub fn new(url: impl ToString) -> Result<Self, DownloaderError> {
        let url = url.to_string();

        let file_name = url
            .replace('\\', "/")
            .replace("://", "");

        let file_name = file_name
            .split('?').next()
            .and_then(|url| {
                url.split('/')
                    .filter(|part| !part.is_empty())
                    .skip(1)
                    .last()
            })
            .unwrap_or("index.html")
            .to_string();

        Ok(Self {
            client: {
                let mut builder = Client::builder()
                    .connect_timeout(STARTUP_CONFIG.general.network.timeout());

                if let Some(proxy) = &STARTUP_CONFIG.general.network.proxy {
                    builder = builder.proxy(proxy.proxy()?);
                }

                builder.build()?
            },

            input_url: url,
            output_file: DATA_FOLDER.join(file_name),
            continue_downloading: false
        })
    }

    #[inline]
    /// Change path to the output file.
    ///
    /// By default used the name from the input URL
    /// stored in the app's data folder.
    pub fn with_output_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.output_file = file.into();

        self
    }

    #[inline]
    /// Change continue downloading state.
    ///
    /// Disabled by default.
    pub fn with_continue_downloading(mut self, state: bool) -> Self {
        self.continue_downloading = state;

        self
    }

    #[inline]
    /// Get path to the output file.
    pub fn output_file(&self) -> &Path {
        self.output_file.as_path()
    }

    /// Start downloading of the file, returning
    /// struct to control the process.
    ///
    /// Input callback handles current and total
    /// number of bytes to download, and a difference
    /// between callback calls.
    ///
    /// This function doesn't block the caller's thread
    /// until the file is downloaded. Async is used to
    /// perform initial HTTP GET request to obtain the
    /// header of the file to get its content size.
    pub async fn download(self, mut progress: impl FnMut(u64, u64, u64) + Send + 'static) -> Result<DownloaderContext, DownloaderError> {
        let Self { client, input_url, output_file, continue_downloading } = self;

        let current = Arc::new(AtomicU64::new(0));
        let total = Arc::new(AtomicU64::new(0));

        // Start a new thread to read incoming data and process it.
        let worker = {
            let current = current.clone();
            let total = total.clone();

            std::thread::spawn(move || {
                RUNTIME.block_on(async move {
                    // Open output file.
                    let file = File::options()
                        .read(true)
                        .write(true)
                        .create(true)
                        .truncate(!continue_downloading)
                        .open(output_file)
                        .await?;

                    // Store its length as downloaded bytes length.
                    let downloaded = file.metadata().await?.len();

                    current.store(downloaded, Ordering::Release);

                    // Add an inner buffer to the output file
                    // to optimize disk writes.
                    let mut file = BufWriter::new(file);

                    file.seek(SeekFrom::Start(downloaded)).await?;

                    // Prepare HTTP request.
                    let request = client.get(input_url)
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

                        // Finish downloading if header says that we've already downloaded all the data
                        if range.contains("*/") {
                            total.store(downloaded, Ordering::Release);

                            return Ok(());
                        }
                    }

                    // HTTP 416 = provided range is greater than the actual
                    // content length (means the file is downloaded).
                    //
                    // Source: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/416
                    if response.status() == 416 {
                        total.store(downloaded, Ordering::Release);

                        return Ok(());
                    }

                    // Try to read the `content-length` HTTP header and if successful,
                    // store its value as the total length of the downloadable content.
                    if let Some(content_length) = response.headers().get("content-length") {
                        let content_length = String::from_utf8_lossy(content_length.as_bytes());

                        if let Ok(content_length) = content_length.parse::<u64>() {
                            total.store(content_length, Ordering::Release);
                        }
                    }

                    // Read chunks of data from the stream
                    // and redirect them to the writer task.
                    while let Some(chunk) = response.chunk().await? {
                        file.write_all(&chunk).await?;

                        let len = chunk.len() as u64;
                        let prev = current.fetch_add(len, Ordering::Relaxed);

                        progress(prev + len, total.load(Ordering::Relaxed), len);

                        if let Some(timeout) = DOWNLOADER_CHUNKS_REQUESTS_TIMEOUT {
                            tokio::time::sleep(timeout).await;
                        }
                    }

                    // Flush remaining buffer to the disk.
                    file.flush().await?;

                    Ok::<_, DownloaderError>(())
                })?;

                Ok::<_, DownloaderError>(())
            })
        };

        Ok(DownloaderContext {
            worker,
            current,
            total
        })
    }
}

#[derive(Debug)]
pub struct DownloaderContext {
    worker: JoinHandle<Result<(), DownloaderError>>,
    current: Arc<AtomicU64>,
    total: Arc<AtomicU64>
}

impl DownloaderContext {
    #[inline]
    /// Get amount of downloaded bytes.
    pub fn current(&self) -> u64 {
        self.current.load(Ordering::Relaxed)
    }

    #[inline]
    /// Get expected total amount of bytes.
    pub fn total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }

    #[inline]
    /// Get downloading progress.
    pub fn progress(&self) -> f32 {
        let current = self.current();
        let total = self.total();

        if current == 0 {
            return 0.0;
        };

        if total == 0 {
            return 1.0;
        }

        current as f32 / total as f32
    }

    #[inline]
    /// Check if downloading has finished.
    ///
    /// Note that it could fail so this doesn't mean that
    /// we've successfully downloaded the file.
    pub fn is_finished(&self) -> bool {
        self.worker.is_finished()
    }

    #[inline]
    /// Wait until the downloader's thread is closed.
    pub fn wait(self) -> Result<(), DownloaderError> {
        self.worker.join().map_err(DownloaderError::ThreadJoin)?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_file() -> Result<(), DownloaderError> {
        let output = Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz")?
            .output_file()
            .to_path_buf();

        assert!(output.ends_with("dxvk-2.4.tar.gz"));

        let output = Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz")?
            .with_output_file("/tmp/amog.us")
            .output_file()
            .to_path_buf();

        assert_eq!(output, PathBuf::from("/tmp/amog.us"));

        let output = Downloader::new("https://github.com")?
            .output_file()
            .to_path_buf();

        assert!(output.ends_with("index.html"));

        Ok(())
    }

    #[tokio::test]
    async fn download() -> Result<(), DownloaderError> {
        let path = std::env::temp_dir().join(".agl-downloader-test");

        let context = Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz")?
            .with_output_file(&path)
            .download(|_, _, _| {})
            .await?;

        while !context.is_finished() {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }

        assert_eq!(context.total(), 9215513);
        assert_eq!(seahash::hash(&std::fs::read(&path)?), 13290421503141924848);

        std::fs::remove_file(path)?;

        Ok(())
    }

    #[tokio::test]
    async fn continue_download() -> Result<(), DownloaderError> {
        let path = std::env::temp_dir().join(".agl-continue-downloader-test");

        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let context_stop = stop.clone();

        let context = Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz")?
            .with_output_file(&path)
            .with_continue_downloading(false)
            .download(move |curr, total, _| {
                if curr > total >> 2 {
                    context_stop.store(true, Ordering::Release);
                }
            })
            .await?;

        while !context.is_finished() {
            if stop.load(Ordering::Acquire) {
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz")?
            .with_output_file(&path)
            .with_continue_downloading(true)
            .download(|_, _, _| {})
            .await?
            .wait()?;

        assert_eq!(seahash::hash(&std::fs::read(&path)?), 13290421503141924848);

        std::fs::remove_file(path)?;

        Ok(())
    }
}
