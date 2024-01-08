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
pub struct DeleteFoldersQueuedTask {
    pub folders: Vec<PathBuf>
}

impl QueuedTask for DeleteFoldersQueuedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        CardInfo::Component {
            name: String::from("delete-folders"),
            title: String::from("Delete folders"),
            developer: String::new()
        }
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        let folders = self.folders.clone();

        Ok(Box::new(DeleteFoldersResolvedTask {
            updater: BasicUpdater::spawn(move |sender| {
                Box::new(move || -> Result<(), anyhow::Error> {
                    sender.send(((), 0, 1))?;

                    for folder in folders {
                        std::fs::remove_dir_all(folder)?;
                    }

                    sender.send(((), 1, 1))?;

                    Ok(())
                })
            })
        }))
    }
}

#[derive(Debug)]
pub struct DeleteFoldersResolvedTask {
    pub updater: BasicUpdater<(), (), anyhow::Error>
}

impl ResolvedTask for DeleteFoldersResolvedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        CardInfo::Component {
            name: String::from("delete-folders"),
            title: String::from("Delete folders"),
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
                BasicStatus::Working(()) => TaskStatus::DeletingFolders,
                BasicStatus::Finished    => TaskStatus::Finished
            }),

            Err(err) => anyhow::bail!(err.to_string())
        }
    }
}
