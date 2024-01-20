use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{
    AtomicU64,
    Ordering
};

use anime_game_core::filesystem::transition::Transition;

use anime_game_core::updater::{
    UpdaterExt,
    BasicUpdater,
    Status as BasicStatus
};

use anime_game_core::network::downloader::{
    DownloaderExt,
    basic::Downloader
};

use crate::ui::components::game_card::CardInfo;

use crate::games;
use crate::games::integrations::standards::prelude::*;

use crate::config;

use super::{
    QueuedTask,
    ResolvedTask,
    TaskStatus
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    PreparingTransition,
    VerifyingFiles,
    RepairingFiles,
    FinishingTransition
}

#[derive(Debug, Clone)]
pub struct VerifyIntegrityQueuedTask {
    pub card_info: CardInfo,
    pub integrity_info: Vec<IntegrityInfo>,
    pub path: PathBuf
}

impl QueuedTask for VerifyIntegrityQueuedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        self.card_info.clone()
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        let config = config::get();

        let game_name = self.card_info.get_name().to_string();
        let game_edition = self.card_info.get_edition().to_string();

        let integrity_info = self.integrity_info.clone();

        let path = self.path.clone();

        Ok(Box::new(VerifyIntegrityResolvedTask {
            card_info: self.card_info.clone(),

            updater: BasicUpdater::spawn(move |sender| {
                Box::new(move || -> Result<(), anyhow::Error> {
                    let game = unsafe {
                        games::get_unsafe(&game_name)
                    };

                    // Check if lua script support custom hashes
                    let has_integrity_hash = game.driver.has_integrity_hash()?;

                    // Create transition

                    sender.send((Status::PreparingTransition, 0, 1))?;

                    let transition = Transition::get_in(
                        format!("verify-integrity:{game_name}:{game_edition}:{:?}", integrity_info),
                        &path,
                        config.general.transitions.path
                    )?;

                    sender.send((Status::PreparingTransition, 1, 1))?;

                    // Verify files

                    let pool = rusty_pool::Builder::new()
                        .name(String::from("verify_files"))
                        .core_size(config.general.threads.number as usize)
                        .build();

                    let queue_size = config.general.threads.max_queue_size as usize;

                    let total = integrity_info.len() as u64;
                    let current = Arc::new(AtomicU64::new(0));

                    let mut tasks = Vec::with_capacity(queue_size);
                    let mut broken_files = Vec::new();

                    sender.send((
                        Status::VerifyingFiles,
                        0,
                        total
                    ))?;

                    // Iterate through integrity files
                    for chunk in integrity_info.chunks(queue_size) {
                        for info in chunk.iter().cloned() {
                            let integrity_file = path.join(&info.file.path);
    
                            // Stop immediately if the file doesn't exist
                            // or its size is different from the remote file
                            if !integrity_file.exists() || integrity_file.metadata()?.len() != info.file.size {
                                broken_files.push(info.file);

                                sender.send((
                                    Status::VerifyingFiles,
                                    current.fetch_add(1, Ordering::Relaxed) + 1,
                                    total
                                ))?;

                                continue;
                            }

                            let current = current.clone();
                            let sender = sender.clone();

                            // Otherwise verifying the file is a heavy task so we put it to the threads pool
                            tasks.push(pool.evaluate(move || -> anyhow::Result<Option<DiffFileDownload>> {
                                // Read existing file
                                let data = std::fs::read(&integrity_file)?;

                                // Get existing file hash
                                let hash = match info.hash {
                                    HashType::Md5 => {
                                        use md5::{Md5, Digest};

                                        format!("{:x}", Md5::digest(&data))
                                    },

                                    HashType::Sha1 => {
                                        use sha1::{Sha1, Digest};

                                        format!("{:x}", Sha1::digest(&data))
                                    },

                                    HashType::Crc32 => {
                                        let mut hasher = crc32fast::Hasher::new();

                                        hasher.update(&data);

                                        hasher.finalize().to_string()
                                    }

                                    HashType::Xxhash32    => format!("{:x}", xxhash_rust::xxh32::xxh32(&data, 0)),
                                    HashType::Xxhash64    => format!("{:x}", xxhash_rust::xxh64::xxh64(&data, 0)),
                                    HashType::Xxhash3_64  => format!("{:x}", xxhash_rust::xxh3::xxh3_64(&data)),
                                    HashType::Xxhash3_128 => format!("{:x}", xxhash_rust::xxh3::xxh3_128(&data)),

                                    HashType::Custom(name) if has_integrity_hash => {
                                        game.driver.integrity_hash(&name, data)?
                                    }

                                    _ => unimplemented!()
                                };

                                sender.send((
                                    Status::VerifyingFiles,
                                    current.fetch_add(1, Ordering::Relaxed) + 1,
                                    total
                                ))?;

                                // Compare existing file hash with integrity info
                                if info.value != hash {
                                    return Ok(Some(info.file));
                                }

                                Ok(None)
                            }));
                        }

                        // Wait for current chunk of files to finish verifying
                        for task in tasks.drain(..) {
                            if let Some(file) = task.await_complete()? {
                                broken_files.push(file);
                            }
                        }
                    }

                    sender.send((
                        Status::VerifyingFiles,
                        total,
                        total
                    ))?;

                    // Repair files

                    let mut tasks = Vec::with_capacity(queue_size);

                    let total = broken_files.len() as u64;
                    let current = Arc::new(AtomicU64::new(0));

                    sender.send((
                        Status::RepairingFiles,
                        0,
                        total
                    ))?;

                    // Go through the broken files list
                    for chunk in broken_files.chunks(queue_size) {
                        for file in chunk.iter().cloned() {
                            let file_path = path.join(&file.path);

                            let current = current.clone();
                            let sender = sender.clone();

                            // Create file repairing task
                            tasks.push(pool.evaluate(move || -> anyhow::Result<()> {
                                // Create parent folder if it doesn't exist
                                if let Some(parent) = file_path.parent() {
                                    if !parent.exists() {
                                        std::fs::create_dir_all(parent)?;
                                    }
                                }

                                // Download the file
                                Downloader::new(file.uri)
                                    .continue_downloading(false)
                                    .download(file_path)?
                                    .wait()?;

                                sender.send((
                                    Status::RepairingFiles,
                                    current.fetch_add(1, Ordering::Relaxed) + 1,
                                    total
                                ))?;

                                Ok(())
                            }));
                        }

                        // Wait for current chunk of files to finish repairing
                        for task in tasks.drain(..) {
                            task.await_complete()?;
                        }
                    }

                    sender.send((
                        Status::RepairingFiles,
                        total,
                        total
                    ))?;

                    // Finish transition

                    sender.send((Status::FinishingTransition, 0, 1))?;

                    transition.finish()?;

                    sender.send((Status::FinishingTransition, 1, 1))?;

                    Ok(())
                })
            })
        }))
    }
}

#[derive(Debug)]
pub struct VerifyIntegrityResolvedTask {
    pub updater: BasicUpdater<Status, (), anyhow::Error>,
    pub card_info: CardInfo
}

impl ResolvedTask for VerifyIntegrityResolvedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        self.card_info.clone()
    }

    #[inline]
    fn is_finished(&mut self) -> bool {
        self.updater.is_finished()
    }

    #[inline]
    fn get_current(&self) -> u64 {
        self.updater.current()
    }

    #[inline]
    fn get_total(&self) -> u64 {
        self.updater.total()
    }

    #[inline]
    fn get_progress(&self) -> f64 {
        self.updater.progress()
    }

    fn get_status(&mut self) -> anyhow::Result<TaskStatus> {
        match self.updater.status() {
            Ok(status) => Ok(match status {
                BasicStatus::Pending => TaskStatus::Pending,

                BasicStatus::Working(Status::PreparingTransition) => TaskStatus::PreparingTransition,
                BasicStatus::Working(Status::VerifyingFiles)      => TaskStatus::VerifyingFiles,
                BasicStatus::Working(Status::RepairingFiles)      => TaskStatus::RepairingFiles,
                BasicStatus::Working(Status::FinishingTransition) => TaskStatus::FinishingTransition,

                BasicStatus::Finished => TaskStatus::Finished
            }),

            Err(err) => anyhow::bail!(err.to_string())
        }
    }
}
