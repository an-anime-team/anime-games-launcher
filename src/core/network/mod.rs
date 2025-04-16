pub mod downloader;

pub mod prelude {
    pub use super::downloader::{
        Downloader,
        DownloaderTask,
        DownloadOptions,
        DownloaderError
    };
}
