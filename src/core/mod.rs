pub mod json;
pub mod network;
pub mod archives;
pub mod filesystem;

pub mod prelude {
    pub use super::json::{
        AsJson,
        AsJsonError
    };

    pub use super::network::downloader::{
        Downloader,
        DownloaderContext,
        DownloaderError
    };

    pub use super::archives::{
        ArchiveExt,
        ArchiveEntry,
        ArchiveExtractionContext,
        get_entries as archive_get_entries,
        get_total_size as archive_get_total_size,
        extract as archive_extract
    };

    pub use super::archives::tar::{
        TarArchive,
        TarArchiveExtractionContext
    };

    pub use super::archives::zip::{
        ZipArchive,
        ZipArchiveExtractionContext
    };

    pub use super::archives::sevenz::{
        SEVENZ_BINARY,
        SevenzArchive,
        SevenzArchiveExtractionContext,
        SevenzArchiveError
    };

    pub use super::filesystem::transactions::Transaction;
}
