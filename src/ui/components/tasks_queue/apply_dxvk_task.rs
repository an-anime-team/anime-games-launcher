use std::path::PathBuf;

use wincompatlib::prelude::*;

use anime_game_core::updater::{
    UpdaterExt,
    BasicUpdater,
    Status as BasicStatus
};

use crate::ui::components::game_card::CardInfo;

use crate::components::wine::Wine;
use crate::components::dxvk::Dxvk;

use super::{
    QueuedTask,
    ResolvedTask,
    TaskStatus
};

#[derive(Debug)]
pub struct ApplyDxvkQueuedTask {
    pub card_info: CardInfo,
    pub dxvk_version: Dxvk,
    pub prefix_path: PathBuf
}

impl QueuedTask for ApplyDxvkQueuedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        self.card_info.clone()
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        let Some(wine) = Wine::from_config()?.to_wincompatlib() else {
            anyhow::bail!("Failed to resolve wincompatlib wine descriptor");
        };

        Ok(Box::new(ApplyDxvkResolvedTask {
            card_info: self.card_info.clone(),

            updater: BasicUpdater::spawn(move |sender| {
                Box::new(move || -> Result<(), anyhow::Error> {
                    // Specify basic wine params

                    let wine = wine
                        .with_arch(WineArch::Win64)
                        .with_loader(WineLoader::Current)
                        .with_prefix(&self.prefix_path);

                    // Apply DXVK

                    sender.send(((), 0, 1))?;

                    wine.install_dxvk(self.dxvk_version.get_folder(), InstallParams {
                        repair_dlls: false,
                        ..InstallParams::default()
                    })?;

                    sender.send(((), 1, 1))?;

                    Ok(())
                })
            })
        }))
    }
}

#[derive(Debug)]
pub struct ApplyDxvkResolvedTask {
    pub updater: BasicUpdater<(), (), anyhow::Error>,
    pub card_info: CardInfo
}

impl ResolvedTask for ApplyDxvkResolvedTask {
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
                BasicStatus::Pending     => TaskStatus::Pending,
                BasicStatus::Working(()) => TaskStatus::InstallingDxvk,
                BasicStatus::Finished    => TaskStatus::Finished
            }),

            Err(err) => anyhow::bail!(err.to_string())
        }
    }
}
