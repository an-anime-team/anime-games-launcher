use std::path::PathBuf;

use anime_game_core::updater::{
    UpdaterExt,
    BasicUpdater,
    Status as BasicStatus
};

use crate::ui::components::game_card::CardInfo;

use super::{
    QueuedTask,
    ResolvedTask,
    TaskStatus
};

#[derive(Debug, Clone)]
pub struct DeleteFilesQueuedTask {
    pub paths: Vec<PathBuf>
}

impl QueuedTask for DeleteFilesQueuedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        CardInfo::Component {
            name: String::from("delete-files"),
            title: String::from("Delete files"),
            developer: String::new()
        }
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        let paths = self.paths.clone();

        Ok(Box::new(DeleteFilesResolvedTask {
            updater: BasicUpdater::spawn(move |sender| {
                Box::new(move || -> Result<(), anyhow::Error> {
                    sender.send(((), 0, 1))?;

                    for path in paths {
                        if path.is_dir() {
                            std::fs::remove_dir_all(path)?;
                        } else if path.is_file() {
                            std::fs::remove_file(path)?;
                        }
                    }

                    sender.send(((), 1, 1))?;

                    Ok(())
                })
            })
        }))
    }
}

#[derive(Debug)]
pub struct DeleteFilesResolvedTask {
    pub updater: BasicUpdater<(), (), anyhow::Error>
}

impl ResolvedTask for DeleteFilesResolvedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        CardInfo::Component {
            name: String::from("delete-files"),
            title: String::from("Delete files"),
            developer: String::new()
        }
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
                BasicStatus::Pending     => TaskStatus::Pending,
                BasicStatus::Working(()) => TaskStatus::DeletingFiles,
                BasicStatus::Finished    => TaskStatus::Finished
            }),

            Err(err) => anyhow::bail!(err.to_string())
        }
    }
}
