use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{
    AtomicU64,
    Ordering
};

use anime_game_core::updater::{
    UpdaterExt,
    BasicUpdater,
    Status as BasicStatus
};

use anime_game_core::archive;
use anime_game_core::filesystem::transition::Transition;

use anime_game_core::network::downloader::{
    DownloaderExt,
    basic::Downloader
};

use crate::ui::components::game_card::CardInfo;

use crate::games;
use crate::games::integrations::standards::diff::DiffInfo;

use crate::config;

use super::{
    QueuedTask,
    ResolvedTask,
    TaskStatus
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DiffOrigin {
    Game,
    Addon {
        group_name: String,
        addon_name: String
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    PreparingTransition,
    Downloading,
    Unpacking,
    RunTransitionCode,
    FinishingTransition,
    RunPostTransitionCode
}

#[derive(Debug, Clone)]
pub struct DownloadDiffQueuedTask {
    pub card_info: CardInfo,
    pub diff_info: DiffInfo,
    pub diff_origin: DiffOrigin,
    pub download_path: PathBuf
}

impl QueuedTask for DownloadDiffQueuedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        self.card_info.clone()
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        let config = config::get();

        let game_name = self.card_info.get_name().to_string();
        let game_edition = self.card_info.get_edition().to_string();

        let diff_info = self.diff_info.clone();
        let diff_origin = self.diff_origin.clone();

        let download_path = self.download_path.clone();

        Ok(Box::new(DownloadDiffResolvedTask {
            card_info: self.card_info.clone(),

            updater: BasicUpdater::spawn(move |sender| {
                Box::new(move || -> Result<(), anyhow::Error> {
                    let game = unsafe {
                        games::get_unsafe(&game_name)
                    };

                    // Create transition

                    sender.send((Status::PreparingTransition, 0, 1))?;

                    let transition = Transition::get_in(
                        format!("download-diff:{game_name}:{game_edition}:{:?}:{:?}", diff_origin, diff_info),
                        &download_path,
                        config.general.transitions.path
                    )?;

                    sender.send((Status::PreparingTransition, 1, 1))?;

                    // Download and extract diff files

                    match diff_info {
                        DiffInfo::Archive { size: _, uri } => {
                            // Download archive

                            let downloader = Downloader::new(uri);

                            let archive = transition.transition_path()
                                .join(downloader.file_name());

                            let mut updater = downloader.download(&archive)?;

                            while !updater.is_finished() {
                                // TODO: add timeouts

                                sender.send((
                                    Status::Downloading,
                                    updater.current(),
                                    updater.total()
                                ))?;
                            }

                            // Extract archive

                            let Some(mut updater) = archive::extract(&archive, transition.transition_path()) else {
                                anyhow::bail!("Failed to extract files from the archive: {:?}", archive);
                            };

                            while let Ok(false) = updater.status() {
                                // TODO: add timeouts

                                sender.send((
                                    Status::Unpacking,
                                    updater.current(),
                                    updater.total()
                                ))?;
                            }

                            // Delete archive

                            std::fs::remove_file(archive)?;
                        }

                        DiffInfo::Segments { size, segments } => {
                            // Download segments

                            let mut archives = vec![];
                            let mut downloaded = 0;

                            for uri in segments {
                                let downloader = Downloader::new(uri);

                                let archive = transition.transition_path()
                                    .join(downloader.file_name());

                                let mut updater = downloader.download(&archive)?;

                                archives.push(archive);

                                while !updater.is_finished() {
                                    // TODO: add timeouts

                                    sender.send((
                                        Status::Downloading,
                                        downloaded + updater.current(),
                                        size
                                        // updater.total()
                                    ))?;
                                }

                                downloaded += updater.total();
                            }

                            // Extract segments

                            let Some(mut updater) = archive::extract(&archives[0], transition.transition_path()) else {
                                anyhow::bail!("Failed to extract files from segmented archive: {:?}", archives[0]);
                            };

                            while let Ok(false) = updater.status() {
                                // TODO: add timeouts

                                sender.send((
                                    Status::Unpacking,
                                    updater.current(),
                                    updater.total()
                                ))?;
                            }

                            // Delete segments

                            for archive in archives {
                                std::fs::remove_file(archive)?;
                            }
                        }

                        DiffInfo::Files { size, files } => {
                            let pool = rusty_pool::Builder::new()
                                .name(String::from("download_files"))
                                .core_size(config.general.threads.number as usize)
                                .build();

                            let queue_size = config.general.threads.max_queue_size as usize;

                            let mut tasks = Vec::with_capacity(queue_size);

                            let downloaded = Arc::new(AtomicU64::new(0));

                            for chunk in files.chunks(queue_size) {
                                for file in chunk {
                                    let download_path = transition.transition_path().join(&file.path);
                                    let download_uri = file.uri.clone();
                                    let file_size = file.size;

                                    let downloaded = downloaded.clone();
                                    let sender = sender.clone();

                                    tasks.push(pool.evaluate(move || -> anyhow::Result<()> {
                                        Downloader::new(download_uri)
                                            .continue_downloading(false)
                                            .download(download_path)?
                                            .wait()?;

                                        let prev = downloaded.fetch_add(file_size, Ordering::Relaxed);

                                        sender.send((
                                            Status::Downloading,
                                            prev + file_size,
                                            size
                                        ))?;

                                        Ok(())
                                    }));
                                }

                                for task in tasks.drain(..) {
                                    task.await_complete()?;
                                }
                            }
                        }
                    }

                    // Run transition code

                    match &diff_origin {
                        DiffOrigin::Game if game.has_game_diff_transition()? => {
                            sender.send((Status::RunTransitionCode, 0, 1))?;

                            game.run_game_diff_transition(
                                transition.transition_path().to_string_lossy(),
                                &game_edition
                            )?;

                            sender.send((Status::RunTransitionCode, 1, 1))?;
                        }

                        DiffOrigin::Addon { group_name, addon_name } if game.has_addons_diff_transition()? => {
                            sender.send((Status::RunTransitionCode, 0, 1))?;

                            game.run_addons_diff_transition(
                                group_name,
                                addon_name,
                                transition.transition_path().to_string_lossy(),
                                &game_edition
                            )?;

                            sender.send((Status::RunTransitionCode, 1, 1))?;
                        }

                        _ => ()
                    }

                    // Finish transition

                    sender.send((Status::FinishingTransition, 0, 1))?;

                    transition.finish()?;

                    sender.send((Status::FinishingTransition, 1, 1))?;

                    // Run post-transition code

                    match &diff_origin {
                        DiffOrigin::Game if game.has_game_diff_post_transition()? => {
                            sender.send((Status::RunPostTransitionCode, 0, 1))?;

                            game.run_game_diff_post_transition(
                                transition.original_path().to_string_lossy(),
                                &game_edition
                            )?;

                            sender.send((Status::RunPostTransitionCode, 1, 1))?;
                        }

                        DiffOrigin::Addon { group_name, addon_name } if game.has_addons_diff_post_transition()? => {
                            sender.send((Status::RunPostTransitionCode, 0, 1))?;

                            game.run_addons_diff_post_transition(
                                group_name,
                                addon_name,
                                transition.original_path().to_string_lossy(),
                                &game_edition
                            )?;

                            sender.send((Status::RunPostTransitionCode, 1, 1))?;
                        }

                        _ => ()
                    }

                    Ok(())
                })
            })
        }))
    }
}

#[derive(Debug)]
pub struct DownloadDiffResolvedTask {
    pub updater: BasicUpdater<Status, (), anyhow::Error>,
    pub card_info: CardInfo
}

impl ResolvedTask for DownloadDiffResolvedTask {
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

                BasicStatus::Working(Status::PreparingTransition)   => TaskStatus::PreparingTransition,
                BasicStatus::Working(Status::Downloading)           => TaskStatus::Downloading,
                BasicStatus::Working(Status::Unpacking)             => TaskStatus::Unpacking,
                BasicStatus::Working(Status::RunTransitionCode)     => TaskStatus::RunTransitionCode,
                BasicStatus::Working(Status::FinishingTransition)   => TaskStatus::FinishingTransition,
                BasicStatus::Working(Status::RunPostTransitionCode) => TaskStatus::RunPostTransitionCode,

                BasicStatus::Finished => TaskStatus::Finished
            }),

            Err(err) => anyhow::bail!(err.to_string())
        }
    }
}
