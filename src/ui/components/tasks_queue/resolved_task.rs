use anime_game_core::updater::UpdaterExt;

use anime_game_core::game::genshin::diff::{
    Updater as GenshinDiffUpdater,
    Status as GenshinDiffStatus
};

use crate::components::{
    Updater as ComponentUpdater,
    Status as ComponentStatus
};

use crate::ui::components::game_card::CardVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// All the possible tasks statuses in one enum
pub enum TaskStatus {
    PreparingTransition,
    Downloading,
    Unpacking,
    FinishingTransition,
    ApplyingHdiffPatches,
    DeletingObsoleteFiles,
    Finished
}

pub enum ResolvedTask {
    DownloadGenshinDiff {
        updater: GenshinDiffUpdater
    },

    DownloadComponent {
        title: String,
        author: String,
        updater: ComponentUpdater
    }
}

impl std::fmt::Debug for ResolvedTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DownloadGenshinDiff { .. } => {
                f.debug_struct("DownloadGenshinDiff")
                    .finish()
            }

            Self::DownloadComponent { title, author, .. } => {
                f.debug_struct("DownloadComponent")
                    .field("title", title)
                    .field("author", author)
                    .finish()
            }
        }
    }
}

impl ResolvedTask {
    /// Get component variant
    pub fn get_variant(&self) -> CardVariant {
        match self {
            Self::DownloadGenshinDiff { .. } => CardVariant::Genshin,

            Self::DownloadComponent { title, author, .. } => CardVariant::Component {
                title: title.clone(),
                author: author.clone()
            }
        }
    }

    /// Get tasked component title
    pub fn get_title(&self) -> String {
        match self {
            Self::DownloadComponent { title, .. } => title.to_owned(),

            _ => self.get_variant().get_title().to_owned()
        }
    }

    /// Get tasked component author
    pub fn get_author(&self) -> String {
        match self {
            Self::DownloadComponent { author, .. } => author.to_owned(),

            _ => self.get_variant().get_author().to_owned()
        }
    }

    /// Check if the task is finished
    pub fn is_finished(&mut self) -> bool {
        match self {
            Self::DownloadGenshinDiff { updater, .. } => updater.is_finished(),
            Self::DownloadComponent { updater, .. } => updater.is_finished(),
        }
    }

    /// Get current task progress
    pub fn get_current(&self) -> usize {
        match self {
            Self::DownloadGenshinDiff { updater, .. } => updater.current(),
            Self::DownloadComponent { updater, .. } => updater.current(),
        }
    }

    /// Get total task progress
    pub fn get_total(&self) -> usize {
        match self {
            Self::DownloadGenshinDiff { updater } => updater.total(),
            Self::DownloadComponent { updater, .. } => updater.total()
        }
    }

    /// Get task completion progress
    pub fn get_progress(&self) -> f64 {
        match self {
            Self::DownloadGenshinDiff { updater } => updater.progress(),
            Self::DownloadComponent { updater, .. } => updater.progress()
        }
    }

    /// Get task status
    pub fn get_unified_status(&mut self) -> anyhow::Result<TaskStatus> {
        match self {
            Self::DownloadGenshinDiff { updater } => {
                match updater.status() {
                    Ok(status) => Ok(match status {
                        GenshinDiffStatus::PreparingTransition   => TaskStatus::PreparingTransition,
                        GenshinDiffStatus::Downloading           => TaskStatus::Downloading,
                        GenshinDiffStatus::Unpacking             => TaskStatus::Unpacking,
                        GenshinDiffStatus::FinishingTransition   => TaskStatus::FinishingTransition,
                        GenshinDiffStatus::ApplyingHdiffPatches  => TaskStatus::ApplyingHdiffPatches,
                        GenshinDiffStatus::DeletingObsoleteFiles => TaskStatus::DeletingObsoleteFiles,
                        GenshinDiffStatus::Finished              => TaskStatus::Finished
                    }),

                    Err(err) => anyhow::bail!(err.to_string())
                }
            }

            Self::DownloadComponent { updater, .. } => {
                match updater.status() {
                    Ok(status) => Ok(match status {
                        ComponentStatus::Downloading => TaskStatus::Downloading,
                        ComponentStatus::Unpacking   => TaskStatus::Unpacking,
                        ComponentStatus::Finished    => TaskStatus::Finished
                    }),

                    Err(err) => anyhow::bail!(err.to_string())
                }
            }
        }
    }
}
