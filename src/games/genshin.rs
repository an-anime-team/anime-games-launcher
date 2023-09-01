use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anime_game_core::updater::Status;

use anime_game_core::game::diff::DiffExt;
use anime_game_core::game::GameExt;
use anime_game_core::game::genshin::{Game, Edition};

use anime_game_core::game::genshin::diff::{
    Diff,
    Status as DiffStatus
};

use anime_game_core::filesystem::DriverExt;

use crate::ui::components::game_card::CardVariant;
use crate::ui::components::tasks_queue::TaskStatus;

use super::{
    RunGameExt,
    DownloadDiffQueuedTask
};

pub struct Genshin {
    driver: Arc<dyn DriverExt>,
    edition: Edition
}

impl From<&Game> for Genshin {
    #[inline]
    fn from(game: &Game) -> Self {
        Self {
            driver: game.get_driver(),
            edition: game.get_edition()
        }
    }
}

impl RunGameExt for Genshin {
    #[inline]
    fn get_game_binary(&self) -> &'static str {
        match self.edition {
            Edition::Global => "GenshinImpact.exe",
            Edition::China  => "YuanShen.exe"
        }
    }

    #[inline]
    fn deploy_game_folder(&self) -> anyhow::Result<PathBuf> {
        Ok(self.driver.deploy()?)
    }

    #[inline]
    fn dismantle_game_folder(&self) -> anyhow::Result<()> {
        Ok(self.driver.dismantle()?)
    }

    #[inline]
    fn get_user_environment(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

impl From<Diff> for DownloadDiffQueuedTask<Diff, <Diff as DiffExt>::Updater> {
    fn from(diff: Diff) -> Self {
        Self {
            variant: CardVariant::Genshin,
            diff,
            get_status: Box::new(|status| match status {
                Ok(status) => Ok(match status {
                    Status::Pending  => TaskStatus::PreparingTransition,
                    Status::Finished => TaskStatus::Finished,

                    Status::Working(status) => match status {
                        DiffStatus::PreparingTransition   => TaskStatus::PreparingTransition,
                        DiffStatus::Downloading           => TaskStatus::Downloading,
                        DiffStatus::Unpacking             => TaskStatus::Unpacking,
                        DiffStatus::RunTransitionCode     => TaskStatus::RunTransitionCode,
                        DiffStatus::FinishingTransition   => TaskStatus::FinishingTransition,
                        DiffStatus::ApplyingHdiffPatches  => TaskStatus::ApplyingHdiffPatches,
                        DiffStatus::DeletingObsoleteFiles => TaskStatus::DeletingObsoleteFiles,
                        DiffStatus::RunPostTransitionCode => TaskStatus::RunPostTransitionCode,
                    }
                }),

                Err(err) => anyhow::bail!(err.to_string())
            })
        }
    }
}
