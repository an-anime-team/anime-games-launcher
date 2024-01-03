use anime_game_core::game::integrity::*;

use anime_game_core::game::genshin::integrity::{
    RepairerStatus,
    VerifyUpdater,
    RepairUpdater
};

use anime_game_core::updater::{
    UpdaterExt,
    Status
};

use crate::config;

use crate::ui::components::game_card::CardInfo;

use super::{
    QueuedTask,
    ResolvedTask,
    TaskStatus
};

impl From<Status<RepairerStatus>> for TaskStatus {
    #[inline]
    fn from(value: Status<RepairerStatus>) -> Self {
        match value {
            Status::Pending  => Self::PreparingTransition,
            Status::Finished => Self::Finished,

            Status::Working(RepairerStatus::PreparingTransition) => Self::PreparingTransition,
            Status::Working(RepairerStatus::FinishingTransition) => Self::FinishingTransition,
            Status::Working(RepairerStatus::RepairingFiles) => Self::RepairingFiles
        }
    }
}

pub struct VerifyIntegrityQueuedTask {
    pub info: CardInfo
}

impl std::fmt::Debug for VerifyIntegrityQueuedTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifyIntegrityQueuedTask")
            .field("info", &self.info)
            .finish()
    }
}

impl QueuedTask for VerifyIntegrityQueuedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        self.info.clone()
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        let config = config::get();

        todo!()

        // let game = match &self.info {
        //     CardVariant::Genshin => config.games.genshin.to_game(),

        //     _ => anyhow::bail!("Card {:?} cannot be represented as the game descriptor", self.variant)
        // };

        // Ok(Box::new(VerifyIntegrityResolvedTask {
        //     variant: self.variant,
        //     verifier: Some(game.verify_files()?),
        //     repairer: None
        // }))
    }
}

pub struct VerifyIntegrityResolvedTask {
    pub info: CardInfo,
    pub verifier: Option<VerifyUpdater>,
    pub repairer: Option<RepairUpdater>
}

impl std::fmt::Debug for VerifyIntegrityResolvedTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifyIntegrityResolvedTask")
            .field("info", &self.info)
            .finish()
    }
}

impl ResolvedTask for VerifyIntegrityResolvedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        self.info.clone()
    }

    fn is_finished(&mut self) -> bool {
        if let Some(verifier) = &mut self.verifier {
            verifier.is_finished()
        }

        else if let Some(repairer) = &mut self.repairer {
            repairer.is_finished()
        }

        else {
            unreachable!()
        }
    }

    #[inline]
    fn get_current(&self) -> u64 {
        if let Some(verifier) = &self.verifier {
            verifier.current()
        }

        else if let Some(repairer) = &self.repairer {
            repairer.current()
        }

        else {
            unreachable!()
        }
    }

    #[inline]
    fn get_total(&self) -> u64 {
        if let Some(verifier) = &self.verifier {
            verifier.total()
        }

        else if let Some(repairer) = &self.repairer {
            repairer.total()
        }

        else {
            unreachable!()
        }
    }

    #[inline]
    fn get_progress(&self) -> f64 {
        if let Some(verifier) = &self.verifier {
            verifier.progress()
        }

        else if let Some(repairer) = &self.repairer {
            repairer.progress()
        }

        else {
            unreachable!()
        }
    }

    #[inline]
    fn get_status(&mut self) -> anyhow::Result<TaskStatus> {
        if let Some(mut verifier) = self.verifier.take() {
            if !verifier.is_finished() {
                let status = verifier.status()
                    .map(|status| if status.is_finished() {
                        TaskStatus::Finished
                    } else {
                        TaskStatus::VerifyingFiles
                    })
                    .map_err(|err| anyhow::anyhow!(err.to_string()));

                self.verifier = Some(verifier);

                status
            }

            else {
                let config = config::get();

                todo!()

                // let game = match &self.info {
                //     CardVariant::Genshin => config.games.genshin.to_game(),

                //     _ => anyhow::bail!("Card {:?} cannot be represented as the game descriptor", self.variant)
                // };

                // self.repairer = Some(game.repair_files(verifier.wait()?)?);

                // Ok(TaskStatus::PreparingTransition)
            }
        }

        else if let Some(repairer) = self.repairer.as_mut() {
            match repairer.status() {
                Ok(status) => Ok(TaskStatus::from(status)),
                Err(err) => anyhow::bail!(err.to_string())
            }
        }

        else {
            unreachable!()
        }
    }
}
