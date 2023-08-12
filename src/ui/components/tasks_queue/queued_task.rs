use anime_game_core::game::diff::DiffExt;

use anime_game_core::game::genshin::diff::Diff as GenshinDiff;

use crate::components::wine::Wine;
use crate::components::dxvk::Dxvk;

use crate::ui::components::game_card::CardVariant;

use super::resolved_task::ResolvedTask;

pub enum QueuedTask {
    DownloadGenshinDiff {
        diff: GenshinDiff
    },

    DownloadWine {
        title: String,
        author: String,
        version: Wine
    },

    DownloadDxvk {
        title: String,
        author: String,
        version: Dxvk
    }
}

impl std::fmt::Debug for QueuedTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DownloadGenshinDiff { .. } => {
                f.debug_struct("DownloadGenshinDiff")
                    .finish()
            }

            Self::DownloadWine { title, author, version } => {
                f.debug_struct("DownloadWine")
                    .field("title", title)
                    .field("author", author)
                    .field("version", version)
                    .finish()
            }

            Self::DownloadDxvk { title, author, version } => {
                f.debug_struct("DownloadDxvk")
                    .field("title", title)
                    .field("author", author)
                    .field("version", version)
                    .finish()
            }
        }
    }
}

impl QueuedTask {
    /// Get component variant
    pub fn get_variant(&self) -> CardVariant {
        match self {
            Self::DownloadGenshinDiff { .. } => CardVariant::Genshin,

            Self::DownloadWine { title, author, .. } |
            Self::DownloadDxvk { title, author, .. } => CardVariant::Component {
                title: title.clone(),
                author: author.clone()
            }
        }
    }

    /// Get tasked component title
    pub fn get_title(&self) -> String {
        match self {
            Self::DownloadWine { title, .. } |
            Self::DownloadDxvk { title, .. } => title.to_owned(),

            _ => self.get_variant().get_title().to_owned()
        }
    }

    /// Get tasked component author
    pub fn get_author(&self) -> String {
        match self {
            Self::DownloadWine { author, .. } |
            Self::DownloadDxvk { author, .. } => author.to_owned(),

            _ => self.get_variant().get_author().to_owned()
        }
    }

    /// Resolve queued task and start downloading stuff
    pub fn resolve(self) -> anyhow::Result<ResolvedTask> {
        match self {
            Self::DownloadGenshinDiff { diff } => {
                let Some(updater) = diff.install() else {
                    anyhow::bail!("Queued genshin diff cannot be installed");
                };

                Ok(ResolvedTask::DownloadGenshinDiff {
                    updater
                })
            }

            Self::DownloadWine { title, author, version } => {
                Ok(ResolvedTask::DownloadComponent {
                    title,
                    author,
                    updater: version.download()?
                })
            }

            Self::DownloadDxvk { title, author, version } => {
                Ok(ResolvedTask::DownloadComponent {
                    title,
                    author,
                    updater: version.download()?
                })
            }
        }
    }
}
