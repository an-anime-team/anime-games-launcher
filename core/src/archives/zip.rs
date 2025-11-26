use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};

use super::*;

pub fn get_entries(
    path: impl AsRef<Path>
) -> Result<Vec<ArchiveEntry>, ArchiveError> {
    let output = Command::new("unzip")
        .arg("-l")
        .arg(path.as_ref())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()?;

    let output = String::from_utf8_lossy(&output.stdout);

    let entries = output
        .trim()
        .split('\n')
        .skip(3)
        .take_while(|line| !line.starts_with("---------"))
        .map(|line| {
            line.split("  ").filter_map(|word| {
                let word = word.trim();

                if word.is_empty() { None } else { Some(word) }
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

pub fn extract(
    archive: impl AsRef<Path>,
    folder: impl AsRef<Path>,
    mut progress: impl FnMut(u64, u64, u64) + Send + 'static
) -> Result<ArchiveExtractor, ArchiveError> {
    let archive = archive.as_ref();
    let folder = folder.as_ref();

    let files = get_entries(archive)?
        .into_iter()
        .map(|entry| (entry.path.to_string_lossy().to_string(), entry.size))
        .collect::<HashMap<String, u64>>();

    let child = Command::new("unzip")
        .stdout(Stdio::piped())
        .arg("-o")
        .arg(archive)
        .arg("-d")
        .arg(folder)
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
                            let file = file.strip_prefix(&prefix).unwrap_or(file).trim_end();

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

    Ok(ArchiveExtractor {
        worker,
        current,
        total
    })
}
