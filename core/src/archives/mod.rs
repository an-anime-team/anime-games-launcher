use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

mod format;

pub use format::ArchiveFormat;

#[cfg(feature = "archives-tar")]
mod tar;

#[cfg(feature = "archives-zip")]
mod zip;

#[cfg(feature = "archives-7z")]
mod sevenz;

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[cfg(feature = "archives-7z")]
    #[error("7z package is not installed")]
    SevenzNotAvailable,

    #[error("unsupported archive format: {0}")]
    UnsupportedFormat(String),

    #[error("failed to extract archive: {0}")]
    ExtractionError(&'static str)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArchiveEntry {
    /// Relative path of the archive entry.
    pub path: PathBuf,

    /// Size of the archive entry.
    ///
    /// Depending on implementation this could either mean compressed or
    /// uncompressed size.
    pub size: u64
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Archive {
    #[cfg(feature = "archives-tar")]
    Tar(PathBuf),

    #[cfg(feature = "archives-zip")]
    Zip(PathBuf),

    #[cfg(feature = "archives-7z")]
    Sevenz(PathBuf)
}

impl Archive {
    /// Open archive from the given file, automatically predict its format.
    ///
    /// Return `None` if format is not supported.
    pub fn open(path: impl Into<PathBuf>) -> Option<Self> {
        let path: PathBuf = path.into();

        let format = ArchiveFormat::from_path(&path)?;

        Self::open_with_format(path, format)
    }

    /// Open archive from the given file with already known format.
    ///
    /// Return `None` if format is not supported.
    pub fn open_with_format(
        path: impl Into<PathBuf>,
        format: ArchiveFormat
    ) -> Option<Self> {
        let path: PathBuf = path.into();

        #[cfg(feature = "tracing")]
        tracing::trace!(?path, ?format, "open archive");

        match format {
            #[cfg(feature = "archives-tar")]
            ArchiveFormat::Tar => Some(Self::Tar(path)),

            #[cfg(feature = "archives-zip")]
            ArchiveFormat::Zip => Some(Self::Zip(path)),

            #[cfg(feature = "archives-7z")]
            ArchiveFormat::Sevenz => Some(Self::Sevenz(path)),

            #[allow(unreachable_patterns)]
            _ => None
        }
    }

    /// Get path of the currently open archive.
    pub const fn path(&self) -> &PathBuf {
        match self {
            #[cfg(feature = "archives-tar")]
            Self::Tar(path) => path,

            #[cfg(feature = "archives-zip")]
            Self::Zip(path) => path,

            #[cfg(feature = "archives-7z")]
            Self::Sevenz(path) => path
        }
    }

    /// Get list of archive entries.
    pub fn get_entries(&self) -> Result<Vec<ArchiveEntry>, ArchiveError> {
        #[cfg(feature = "tracing")]
        tracing::trace!(path = ?self.path(), "get archive entries");

        match self {
            #[cfg(feature = "archives-tar")]
            Self::Tar(archive) => tar::get_entries(archive),

            #[cfg(feature = "archives-zip")]
            Self::Zip(archive) => zip::get_entries(archive),

            #[cfg(feature = "archives-7z")]
            Self::Sevenz(archive) => sevenz::get_entries(archive)
        }
    }

    /// Extract archive's content to a folder.
    #[inline]
    pub fn extract(
        &self,
        folder: impl AsRef<Path>
    ) -> Result<ArchiveExtractor, ArchiveError> {
        self.extract_with_progress(folder, |_, _, _| {})
    }

    /// Extract archive's content to a folder and report `(curr, total, diff)`
    /// bytes using the `progress` callback.
    pub fn extract_with_progress(
        &self,
        folder: impl AsRef<Path>,
        progress: impl FnMut(u64, u64, u64) + Send + 'static
    ) -> Result<ArchiveExtractor, ArchiveError> {
        let folder = folder.as_ref();

        #[cfg(feature = "tracing")]
        tracing::trace!(path = ?self.path(), output = ?folder, "extract archive");

        match self {
            #[cfg(feature = "archives-tar")]
            Self::Tar(archive) => tar::extract(archive, folder, progress),

            #[cfg(feature = "archives-zip")]
            Self::Zip(archive) => zip::extract(archive, folder, progress),

            #[cfg(feature = "archives-7z")]
            Self::Sevenz(archive) => sevenz::extract(archive, folder, progress)
        }
    }
}

pub struct ArchiveExtractor {
    pub(crate) worker: JoinHandle<()>,
    pub(crate) current: Arc<AtomicU64>,
    pub(crate) total: u64
}

impl ArchiveExtractor {
    #[inline]
    pub fn current(&self) -> u64 {
        self.current.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub const fn total(&self) -> u64 {
        self.total
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.worker.is_finished()
    }

    pub fn wait(self) -> Result<(), ArchiveError> {
        self.worker
            .join()
            .map_err(|_| ArchiveError::ExtractionError("failed to join the thread"))?;

        Ok(())
    }
}
