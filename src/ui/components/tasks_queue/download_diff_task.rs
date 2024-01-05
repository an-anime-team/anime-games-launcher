use std::sync::Arc;

use anime_game_core::updater::{
    UpdaterExt,
    BasicUpdater,
    Status as BasicStatus
};

use anime_game_core::network::downloader::{
    DownloaderExt,
    basic::Downloader
};

use anime_game_core::filesystem::DriverExt;
use anime_game_core::archive;

use crate::ui::components::game_card::CardInfo;

use crate::games;
use crate::games::integrations::standards::diff::DiffInfo;

use super::{
    QueuedTask,
    ResolvedTask,
    TaskStatus
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    PreparingTransition,
    Downloading,
    Unpacking,
    RunTransitionCode,
    FinishingTransition,
    RunPostTransitionCode
}

pub struct DownloadDiffQueuedTask {
    pub driver: Arc<dyn DriverExt>,
    pub card_info: CardInfo,
    pub diff_info: DiffInfo
}

impl std::fmt::Debug for DownloadDiffQueuedTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DownloadDiffQueuedTask")
            .field("card_info", &self.card_info)
            .field("diff_info", &self.diff_info)
            .finish()
    }
}

impl QueuedTask for DownloadDiffQueuedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        self.card_info.clone()
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        let driver = self.driver.clone();
        let game_name = self.card_info.get_name().to_string();
        let game_edition = self.card_info.get_edition().to_string();
        let diff_info = self.diff_info.clone();

        Ok(Box::new(DownloadDiffResolvedTask {
            card_info: self.card_info.clone(),

            updater: BasicUpdater::spawn(move |sender| {
                Box::new(move || -> Result<(), anyhow::Error> {
                    let Some(game) = games::get(&game_name)? else {
                        anyhow::bail!("Couldn't find {game_name} integration script");
                    };

                    // Create transition

                    sender.send((Status::PreparingTransition, 0, 1))?;

                    let transition_name = format!("download-diff:{game_name}:{:?}", diff_info); // TODO: add more metadata to the transition name
                    let transition_path = driver.create_transition(&transition_name)?;

                    sender.send((Status::PreparingTransition, 1, 1))?;

                    // Download and extract diff files

                    match diff_info {
                        DiffInfo::Archive { size, uri } => {
                            // Download archive

                            let downloader = Downloader::new(uri);

                            let archive = transition_path.join(downloader.file_name());

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

                            let Some(mut updater) = archive::extract(&archive, &transition_path) else {
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

                                let archive = transition_path.join(downloader.file_name());

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

                            let Some(mut updater) = archive::extract(&archives[0], &transition_path) else {
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
                            todo!()
                        }
                    }

                    // Run transition code

                    if game.has_game_diff_transition()? {
                        sender.send((Status::RunTransitionCode, 0, 1))?;

                        game.run_game_diff_transition(transition_path.to_string_lossy(), &game_edition)?;

                        sender.send((Status::RunTransitionCode, 1, 1))?;
                    }

                    // Finish transition

                    sender.send((Status::FinishingTransition, 0, 1))?;

                    driver.finish_transition(&transition_name)?;

                    sender.send((Status::FinishingTransition, 1, 1))?;

                    // Run post-transition code

                    if game.has_game_diff_post_transition()? {
                        sender.send((Status::RunPostTransitionCode, 0, 1))?;

                        let path = driver.deploy()?;

                        game.run_game_diff_post_transition(path.to_string_lossy(), &game_edition)?;

                        driver.dismantle()?;

                        sender.send((Status::RunPostTransitionCode, 1, 1))?;
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
