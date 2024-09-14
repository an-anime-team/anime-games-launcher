use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread::JoinHandle;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use std::io::{BufReader, BufRead};

use super::*;

/// Simple wrapper around `unzip` binary.
pub struct ZipArchive {
    path: PathBuf
}

impl ArchiveExt for ZipArchive {
    type Extractor = ZipArchiveExtractionContext;
    type Error = std::io::Error;

    fn open(path: impl AsRef<Path>) -> Result<Self, Self::Error> where Self: Sized {
        Ok(Self {
            path: path.as_ref()
                .to_path_buf()
        })
    }

    fn get_entries(&self) -> Result<Vec<ArchiveEntry>, Self::Error> {
        let output = Command::new("unzip")
            .arg("-l")
            .arg(&self.path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()?;

        let output = String::from_utf8_lossy(&output.stdout);

        let entries = output.trim()
            .split('\n')
            .skip(3)
            .take_while(|line| !line.starts_with("---------"))
            .map(|line| {
                line.split("  ").filter_map(|word| {
                    let word = word.trim();

                    if word.is_empty() {
                        None
                    } else {
                        Some(word)
                    }
                })
            })
            .flat_map(|mut words| {
                let size = words.next().map(|size| size.parse());
                let path = words.last().map(PathBuf::from);

                if let (Some(path), Some(Ok(size))) = (path, size) {
                    Some(ArchiveEntry {
                        path,
                        size
                    })
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

        let files = HashMap::<String, u64>::from_iter({
            self.get_entries()?
                .into_iter()
                .map(|entry| (
                    entry.path.to_string_lossy().to_string(),
                    entry.size
                ))
        });

        let child = Command::new("unzip")
            .stdout(Stdio::piped())
            .arg("-o")
            .arg(self.path.as_path())
            .arg("-d")
            .arg(folder.as_path())
            .spawn()?;

        let prefix = format!("{}/", folder.to_string_lossy());

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

                        // Strip 'Archive: ...' and other top-level info messages.
                        if let Some(line) = line.strip_prefix(' ') {
                            // extracting: sus/1001.pck
                            // inflating: sus/3x.webp
                            // linking: sus/3x.symlink          -> 3x.webp
                            if let Some((_, file)) = line.split_once(": ") {
                                // Remove output directory prefix.
                                let file = file.strip_prefix(&prefix)
                                    .unwrap_or(file)
                                    .trim_end();

                                // If we have this file listed in the entries
                                // sum its size with the current counter.
                                if let Some(size) = files.get(file).copied() {
                                    let prev = current.fetch_add(size, Ordering::Relaxed);

                                    progress(prev + size, total, prev);
                                }
                            }
                        }
                    }
                }
            })
        };

        Ok(ZipArchiveExtractionContext {
            worker,
            current,
            total
        })
    }
}

pub struct ZipArchiveExtractionContext {
    worker: JoinHandle<()>,
    current: Arc<AtomicU64>,
    total: u64
}

impl ArchiveExtractionContext for ZipArchiveExtractionContext {
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

    fn wait(self) -> Result<(), Self::Error> {
        self.worker.join()?;

        Ok(())
    }
}
