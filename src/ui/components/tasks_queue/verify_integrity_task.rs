use anime_game_core::game::integrity::*;
use anime_game_core::updater::UpdaterExt;

use crate::config;

use crate::ui::components::game_card::CardVariant;

use super::{
    QueuedTask,
    ResolvedTask,
    TaskStatus
};

impl From<BasicRepairerUpdaterStatus> for TaskStatus {
    #[inline]
    fn from(value: BasicRepairerUpdaterStatus) -> Self {
        match value {
            BasicRepairerUpdaterStatus::PreparingTransition => Self::PreparingTransition,
            BasicRepairerUpdaterStatus::FinishingTransition => Self::FinishingTransition,
            BasicRepairerUpdaterStatus::RepairingFiles => Self::RepairingFiles,
            BasicRepairerUpdaterStatus::Finished => Self::Finished
        }
    }
}

pub struct VerifyIntegrityQueuedTask {
    pub variant: CardVariant
}

impl std::fmt::Debug for VerifyIntegrityQueuedTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifyIntegrityQueuedTask")
            .field("variant", &self.variant.get_title())
            .finish()
    }
}

impl QueuedTask for VerifyIntegrityQueuedTask {
    #[inline]
    fn get_variant(&self) -> CardVariant {
        self.variant.clone()
    }

    #[inline]
    fn get_title(&self) -> &str {
        self.variant.get_title()
    }

    #[inline]
    fn get_author(&self) -> &str {
        self.variant.get_author()
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        let config = config::get();

        let game = match &self.variant {
            CardVariant::Genshin => config.games.genshin.to_game(),

            _ => anyhow::bail!("Card {:?} cannot be represented as the game descriptor", self.variant)
        };

        Ok(Box::new(VerifyIntegrityResolvedTask {
            variant: self.variant,
            verifier: Some(game.verify_files()?),
            repairer: None
        }))
    }
}

pub struct VerifyIntegrityResolvedTask {
    pub variant: CardVariant,
    pub verifier: Option<BasicVerifierUpdater>,
    pub repairer: Option<BasicRepairerUpdater>
}

impl std::fmt::Debug for VerifyIntegrityResolvedTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifyIntegrityResolvedTask")
            .field("variant", &self.variant.get_title())
            .finish()
    }
}

impl ResolvedTask for VerifyIntegrityResolvedTask {
    #[inline]
    fn get_variant(&self) -> CardVariant {
        self.variant.clone()
    }

    #[inline]
    fn get_title(&self) -> &str {
        self.variant.get_title()
    }

    #[inline]
    fn get_author(&self) -> &str {
        self.variant.get_author()
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
                    .map(|status| if !status {
                        TaskStatus::VerifyingFiles
                    } else {
                        TaskStatus::Finished
                    })
                    .map_err(|err| anyhow::anyhow!(err.to_string()));

                self.verifier = Some(verifier);

                status
            }

            else {
                let config = config::get();

                let game = match &self.variant {
                    CardVariant::Genshin => config.games.genshin.to_game(),

                    _ => anyhow::bail!("Card {:?} cannot be represented as the game descriptor", self.variant)
                };

                self.repairer = Some(game.repair_files(verifier.wait()?)?);

                Ok(TaskStatus::PreparingTransition)
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
