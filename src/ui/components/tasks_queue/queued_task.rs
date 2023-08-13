use std::cell::Cell;
use std::path::PathBuf;

use anime_game_core::game::diff::DiffExt;

use anime_game_core::game::genshin::diff::Diff as GenshinDiff;

use wincompatlib::wine::ext::{
    WineBootExt,
    WineFontsExt,
    Font
};

use crate::components::wine::Wine;
use crate::components::dxvk::Dxvk;

use crate::ui::components::game_card::CardVariant;

use super::resolved_task::ResolvedTask;

use super::create_prefix_task::{
    Updater as CreatePrefixUpdater,
    Status as CreatePrefixStatus
};

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
    },

    CreatePrefix {
        path: PathBuf,
        install_corefonts: bool
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

            Self::CreatePrefix { path, install_corefonts } => {
                f.debug_struct("CreatePrefix")
                    .field("path", path)
                    .field("install_corefonts", install_corefonts)
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
            },

            Self::CreatePrefix { .. } => CardVariant::Component {
                title: String::from("Wine prefix"),
                author: String::new()
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

            Self::CreatePrefix { path, install_corefonts } => {
                let (sender, receiver) = flume::unbounded();

                let Some(wine) = Wine::from_config()?.to_wincompatlib() else {
                    anyhow::bail!("Failed to resolve wincompatlib wine descriptor");
                };

                Ok(ResolvedTask::CreatePrefix {
                    updater: CreatePrefixUpdater {
                        status: Cell::new(CreatePrefixStatus::CreatingPrefix),
                        current: Cell::new(0),
                        total: Cell::new(1), // To prevent division by 0

                        worker_result: None,
                        updater: receiver,

                        worker: Some(std::thread::spawn(move || -> anyhow::Result<()> {
                            // Create wine prefix

                            if path.exists() {
                                wine.update_prefix(Some(&path))?;
                            } else {
                                wine.init_prefix(Some(&path))?;
                            }

                            // Install fonts

                            if install_corefonts {
                                let total_fonts = Font::iterator().into_iter().count() as u64;

                                for (i, font) in Font::iterator().into_iter().enumerate() {
                                    if !font.is_installed(&path) {
                                        wine.install_font(font)?;
                                    }

                                    sender.send((CreatePrefixStatus::InstallingFont(font), i as u64 + 1, total_fonts))?;
                                }
                            }

                            // Finish downloading

                            sender.send((CreatePrefixStatus::Finished, 0, 1))?;

                            Ok(())
                        }))
                    }
                })
            }
        }
    }
}
