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
    let output = Command::new("tar")
        .arg("-tvf")
        .arg(path.as_ref())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()?;

    let output = String::from_utf8_lossy(&output.stdout);

    let entries = output
        .trim()
        .split('\n')
        .take_while(|line| !line.starts_with("---------"))
        .map(|line| {
            line.split(' ').filter_map(|word| {
                let word = word.trim();

                if word.is_empty() { None } else { Some(word) }
            })
        })
        .flat_map(|mut words| {
            let flags = words.next();
            let size = words.nth(1).map(|size| size.parse());
            let path = words.next_back().map(PathBuf::from);

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

pub fn extract(
    archive: impl AsRef<Path>,
    folder: impl AsRef<Path>,
    mut progress: impl FnMut(u64, u64, u64) + Send + 'static
) -> Result<ArchiveExtractor, ArchiveError> {
    let archive = archive.as_ref();
    let folder = folder.as_ref();

    // Create output directory because tar doesn't do it automatically.
    if !folder.is_dir() {
        std::fs::create_dir_all(folder)?;
    }

    let files = get_entries(archive)?
        .into_iter()
        .map(|entry| (entry.path.to_string_lossy().to_string(), entry.size))
        .collect::<HashMap<String, u64>>();

    let child = Command::new("tar")
        .stdout(Stdio::piped())
        .arg("-xhvf")
        .arg(archive)
        .arg("-C")
        .arg(folder)
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

    Ok(ArchiveExtractor {
        worker,
        current,
        total
    })
}

// #[cfg(test)]
// mod tests {
//     use crate::network::downloader::{
//         Downloader,
//         DownloaderError
//     };

//     use super::*;

//     async fn get_archive() -> Result<TarArchive, DownloaderError> {
//         let path = std::env::temp_dir().join(".wineyard-core-tar-test");

//         if !path.exists() {
//             let downloader = Downloader::default();

//             let task = downloader.download(
//                 "https://github.com/doitsujin/dxvk/releases/download/v2.6.1/dxvk-2.6.1.tar.gz",
//                 &path
//             );

//             task.wait().await?;
//         }

//         Ok(TarArchive::open(path).unwrap())
//     }

//     #[tokio::test]
//     async fn entries() -> Result<(), DownloaderError> {
//         let entries = get_archive()
//             .await?
//             .get_entries()
//             .unwrap();

//         assert_eq!(entries.len(), 13);

//         assert_eq!(
//             entries.iter().map(|entry| entry.size).sum::<u64>(),
//             28119180
//         );

//         assert!(
//             entries
//                 .iter()
//                 .any(|entry| entry.path == PathBuf::from("dxvk-2.6.1/x64/d3d11.dll"))
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn extract() -> Result<(), DownloaderError> {
//         let path = std::env::temp_dir().join(".wineyard-core-tar-test-folder");

//         if path.exists() {
//             std::fs::remove_dir_all(&path)?;
//         }

//         get_archive()
//             .await?
//             .extract(&path)
//             .unwrap()
//             .wait()
//             .unwrap();

//         assert!(
//             path.join("dxvk-2.6.1")
//                 .join("x64")
//                 .join("d3d11.dll")
//                 .exists()
//         );

//         Ok(())
//     }
// }
