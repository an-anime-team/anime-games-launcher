use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread::JoinHandle;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use std::io::{BufReader, BufRead};

use super::*;

/// Simple wrapper around `tar` binary.
pub struct TarArchive {
    path: PathBuf
}

impl ArchiveExt for TarArchive {
    type Extractor = TarArchiveExtractionContext;
    type Error = std::io::Error;

    fn open(path: impl AsRef<Path>) -> Result<Self, Self::Error> where Self: Sized {
        Ok(Self {
            path: path.as_ref()
                .to_path_buf()
        })
    }

    fn get_entries(&self) -> Result<Vec<ArchiveEntry>, Self::Error> {
        let output = Command::new("tar")
            .arg("-tvf")
            .arg(&self.path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()?;

        let output = String::from_utf8_lossy(&output.stdout);

        let entries = output.trim()
            .split('\n')
            .take_while(|line| !line.starts_with("---------"))
            .map(|line| {
                line.split(' ').filter_map(|word| {
                    let word = word.trim();

                    if word.is_empty() {
                        None
                    } else {
                        Some(word)
                    }
                })
            })
            .flat_map(|mut words| {
                let flags = words.next();
                let size = words.nth(1).map(|size| size.parse());
                let path = words.last().map(PathBuf::from);

                if let (Some(flags), Some(path), Some(Ok(size))) = (flags, path, size) {
                    // Skip symlinks
                    // FIXME: parse them as well
                    if flags.starts_with('l') {
                        None
                    } else {
                        Some(ArchiveEntry {
                            path,
                            size
                        })
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok(entries)
    }

    fn extract(&self, folder: impl AsRef<Path>, mut progress: impl FnMut(u64, u64, u64) + Send + 'static) -> Result<Self::Extractor, Self::Error> {
        let folder = folder.as_ref()
            .to_path_buf();

        // Create output directory because
        // tar doesn't do it automatically.
        std::fs::create_dir_all(&folder)?;

        let files = HashMap::<String, u64>::from_iter({
            self.get_entries()?
                .into_iter()
                .map(|entry| (
                    entry.path.to_string_lossy().to_string(),
                    entry.size
                ))
        });

        let child = Command::new("tar")
            .stdout(Stdio::piped())
            .arg("-xhvf")
            .arg(self.path.as_path())
            .arg("-C")
            .arg(folder.as_path())
            .spawn()?;

        let current = Arc::new(AtomicU64::new(0));
        let total = files.values().sum::<u64>();

        let worker = {
            let current = current.clone();

            std::thread::spawn(move || {
                if let Some(stdout) = child.stdout {
                    let stdout = BufReader::new(stdout);

                    for line in stdout.lines() {
                        let Ok(line) = line else {
                            // TODO: throw the error to the context.
                            break;
                        };

                        // If we have this file listed in the entries
                        // sum its size with the current counter.
                        if let Some(size) = files.get(&line).copied() {
                            let prev = current.fetch_add(size, Ordering::Relaxed);

                            progress(prev + size, total, prev);
                        }
                    }
                }
            })
        };

        Ok(TarArchiveExtractionContext {
            worker,
            current,
            total
        })
    }
}

pub struct TarArchiveExtractionContext {
    worker: JoinHandle<()>,
    current: Arc<AtomicU64>,
    total: u64
}

impl ArchiveExtractionContext for TarArchiveExtractionContext {
    type Error = Box<dyn std::any::Any + Send + 'static>;

    #[inline]
    fn current(&self) -> u64 {
        self.current.load(Ordering::Relaxed)
    }

    #[inline]
    fn total(&self) -> u64 {
        self.total
    }

    #[inline]
    fn is_finished(&self) -> bool {
        self.worker.is_finished()
    }

    #[inline]
    fn wait(self) -> Result<(), Self::Error> {
        self.worker.join()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::core::prelude::*;

    use super::*;

    async fn get_archive() -> Result<TarArchive, DownloaderError> {
        let path = std::env::temp_dir().join(".agl-tar-test");

        if !path.exists() {
            Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz")?
                .with_output_file(&path)
                .download(|_, _, _| {})
                .await?
                .wait()?;
        }

        Ok(TarArchive::open(path)?)
    }

    #[tokio::test]
    async fn entries() -> Result<(), DownloaderError> {
        let entries = get_archive().await?
            .get_entries()?;

        assert_eq!(entries.len(), 13);
        assert_eq!(entries.iter().map(|entry| entry.size).sum::<u64>(), 25579660);
        assert!(entries.iter().any(|entry| entry.path == PathBuf::from("dxvk-2.4/x64/d3d10core.dll")));

        Ok(())
    }

    #[tokio::test]
    async fn extract() -> Result<(), DownloaderError> {
        let path = std::env::temp_dir().join(".agl-tar-test-folder");

        if path.exists() {
            std::fs::remove_dir_all(&path)?;
        }

        get_archive().await?
            .extract(&path, |_, _, _| {})?
            .wait().unwrap();

        assert!(path.join("dxvk-2.4")
            .join("x64")
            .join("d3d10core.dll")
            .exists());

        Ok(())
    }
}
