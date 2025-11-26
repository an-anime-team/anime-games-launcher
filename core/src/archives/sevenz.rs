use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};

use super::*;

lazy_static::lazy_static! {
    /// Name of the 7z binary installed on the system.
    pub static ref SEVENZ_BINARY: Option<&'static str> = {
        for binary in ["7z", "7za", "7zz"] {
            let result = Command::new(binary)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output();

            if result.is_ok() {
                return Some(binary);
            }
        }

        None
    };
}

pub fn get_entries(
    path: impl AsRef<Path>
) -> Result<Vec<ArchiveEntry>, ArchiveError> {
    let Some(binary) = SEVENZ_BINARY.as_ref() else {
        return Err(ArchiveError::SevenzNotAvailable);
    };

    let output = Command::new(binary)
        .arg("l")
        .arg(path.as_ref())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()?;

    let output = String::from_utf8_lossy(&output.stdout);

    let output = output
        .split("-------------------")
        .skip(1)
        .collect::<Vec<_>>();

    let mut output = output[..output.len() - 1].join("-------------------");

    // In some cases 7z can report two ending sequences instead of one:
    //
    // ```
    // ------------------- ----- ------------ ------------  ------------------------
    // 2023-09-15 10:20:44        66677218871  65387995385  13810 files, 81 folders
    //
    // ------------------- ----- ------------ ------------  ------------------------
    // 2023-09-15 10:20:44        66677218871  65387995385  13810 files, 81 folders
    // ```
    //
    // This should filter this case.
    if let Some((files_list, _)) = output.split_once("\n-------------------") {
        output = files_list.to_string();
    }

    let entries = output
        .split('\n')
        .filter(|line| !line.starts_with('-') && !line.starts_with(" -"))
        .map(|line| {
            line.split("  ").filter_map(|word| {
                let word = word.trim();

                if word.is_empty() { None } else { Some(word) }
            })
        })
        .flat_map(|mut words| {
            let size = words.nth(1).map(|size| size.parse());
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
        .collect();

    Ok(entries)
}

pub fn extract(
    archive: impl AsRef<Path>,
    folder: impl AsRef<Path>,
    mut progress: impl FnMut(u64, u64, u64) + Send + 'static
) -> Result<ArchiveExtractor, ArchiveError> {
    let Some(binary) = SEVENZ_BINARY.as_ref() else {
        return Err(ArchiveError::SevenzNotAvailable);
    };

    let archive = archive.as_ref();
    let folder = folder.as_ref();

    let files = get_entries(archive)?
        .into_iter()
        .map(|entry| (entry.path.to_string_lossy().to_string(), entry.size))
        .collect::<HashMap<String, u64>>();

    let child = Command::new(binary)
        .stdout(Stdio::piped())
        .arg("x")
        .arg(archive)
        .arg(format!("-o{}", folder.to_string_lossy()))
        .arg("-aoa")
        .arg("-bb1")
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

                    if let Some(file) = line.strip_prefix("- ") {
                        // If we have this file listed in the entries
                        // sum its size with the current counter.
                        if let Some(size) = files.get(file).copied() {
                            let prev = current.fetch_add(size, Ordering::Relaxed);

                            progress(prev + size, total, prev);
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
