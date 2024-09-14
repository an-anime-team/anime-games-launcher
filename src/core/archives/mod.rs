use std::path::{Path, PathBuf};

pub mod tar;
pub mod zip;
pub mod sevenz;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArchiveEntry {
    /// Relative path of the archive entry.
    pub path: PathBuf,

    /// Size of the archive entry.
    /// Depending on implementation this could
    /// either mean compressed or uncompressed size.
    pub size: u64
}

pub trait ArchiveExtractionContext {
    type Error;

    /// Get amount of extracted bytes.
    fn current(&self) -> u64;

    /// Get total amount of bytes to extract.
    fn total(&self) -> u64;

    #[inline]
    /// Get files extraction progress.
    fn progress(&self) -> f32 {
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

    /// Check if extraction has finished.
    ///
    /// Note that it could fail so this doesn't mean that
    /// we've successfully extracted all the files.
    fn is_finished(&self) -> bool;

    /// Wait until the extraction is completed.
    fn wait(self) -> Result<(), Self::Error>;
}

pub trait ArchiveExt {
    type Extractor: ArchiveExtractionContext;
    type Error;

    /// Open archive.
    fn open(path: impl AsRef<Path>) -> Result<Self, Self::Error> where Self: Sized;

    /// Get list of entries in the archive.
    fn get_entries(&self) -> Result<Vec<ArchiveEntry>, Self::Error>;

    /// Get total size of all the entries in the archive.
    fn total_size(&self) -> Result<u64, Self::Error> {
        Ok(self.get_entries()?
            .into_iter()
            .map(|entry| entry.size)
            .sum())
    }

    /// Extract archive's content to a folder, using
    /// provided callback to handle the progress.
    ///
    /// Callback accepts currently extracted amount of
    /// bytes, total expected amount and a diff between
    /// calls.
    fn extract(&self, folder: impl AsRef<Path>, progress: impl FnMut(u64, u64, u64) + Send + 'static) -> Result<Self::Extractor, Self::Error>;
}

enum ArchiveFormat {
    Tar,
    Zip,
    Sevenz
}

fn get_format(path: impl AsRef<Path>) -> Option<ArchiveFormat> {
    let path = path.as_ref()
        .as_os_str()
        .to_string_lossy();

    if path.ends_with(".tar.xz") || path.ends_with(".tar.gz") || path.ends_with(".tar.bz2") || path.ends_with(".tar") {
        Some(ArchiveFormat::Tar)
    }

    else if path.ends_with(".zip") {
        Some(ArchiveFormat::Zip)
    }

    else if path.ends_with(".7z") | path.ends_with(".zip.001") || path.ends_with(".7z.001") {
        Some(ArchiveFormat::Sevenz)
    }

    else {
        None
    }
}

/// Get list of entries of an archive.
///
/// Utility function that automatically predicts
/// the format of the archive and needed struct
/// to process it.
pub fn get_entries(path: impl AsRef<Path>) -> anyhow::Result<Vec<ArchiveEntry>> {
    let entries = match get_format(path.as_ref()) {
        Some(ArchiveFormat::Tar) => tar::TarArchive::open(path)
            .and_then(|archive| archive.get_entries())?,

        Some(ArchiveFormat::Zip) => zip::ZipArchive::open(path)
            .and_then(|archive| archive.get_entries())?,

        Some(ArchiveFormat::Sevenz) => sevenz::SevenzArchive::open(path)
            .and_then(|archive| archive.get_entries())?,

        None => anyhow::bail!("Unknown archive format: {:?}", path.as_ref())
    };

    Ok(entries)
}

/// Get total size of files in the archive.
///
/// Utility function that automatically predicts
/// the format of the archive and needed struct
/// to process it.
pub fn get_total_size(path: impl AsRef<Path>) -> anyhow::Result<u64> {
    let total_size = match get_format(path.as_ref()) {
        Some(ArchiveFormat::Tar) => tar::TarArchive::open(path)
            .and_then(|archive| archive.total_size())?,

        Some(ArchiveFormat::Zip) => zip::ZipArchive::open(path)
            .and_then(|archive| archive.total_size())?,

        Some(ArchiveFormat::Sevenz) => sevenz::SevenzArchive::open(path)
            .and_then(|archive| archive.total_size())?,

        None => anyhow::bail!("Unknown archive format: {:?}", path.as_ref())
    };

    Ok(total_size)
}

/// Extract files from the archive.
///
/// Utility function that automatically predicts
/// the format of the archive and needed struct
/// to process it.
///
/// This function will freeze the current thread
/// until the archive is fully extracted.
pub fn extract(path: impl AsRef<Path>, folder: impl AsRef<Path>, progress: impl FnMut(u64, u64, u64) + Send + 'static) -> anyhow::Result<()> {
    match get_format(path.as_ref()) {
        Some(ArchiveFormat::Tar) => {
            tar::TarArchive::open(path)
                .and_then(|archive| archive.extract(folder, progress))?
                .wait()
                .map_err(|err| anyhow::anyhow!("Failed to extract archive: {err:?}"))?;
        }

        Some(ArchiveFormat::Zip) => {
            zip::ZipArchive::open(path)
                .and_then(|archive| archive.extract(folder, progress))?
                .wait()
                .map_err(|err| anyhow::anyhow!("Failed to extract archive: {err:?}"))?;
        }

        Some(ArchiveFormat::Sevenz) => {
            sevenz::SevenzArchive::open(path)
                .and_then(|archive| archive.extract(folder, progress))?
                .wait()
                .map_err(|err| anyhow::anyhow!("Failed to extract archive: {err:?}"))?;
        }

        None => anyhow::bail!("Unknown archive format: {:?}", path.as_ref())
    }

    Ok(())
}
