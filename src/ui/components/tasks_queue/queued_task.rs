use std::cell::Cell;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

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

                            // TODO: apply DXVK

                            // Install fonts

                            if install_corefonts {
                                let wine_arc = Arc::new(wine);
                                let path_arc = Arc::new(path);

                                let total_fonts = Font::iterator().into_iter().count() as u64;
                                let font_queue = Arc::new(Mutex::new(Font::iterator().into_iter().collect::<Vec<Font>>()));
                                let installed_fonts = Arc::new(AtomicU64::new(0));

                                let thread_count = 8; // TODO: determine thread count using something better than a magic number
                                let mut threads = Vec::with_capacity(thread_count);

                                sender.send((CreatePrefixStatus::InstallingFonts, 0, total_fonts))?;

                                for _ in 0..thread_count {
                                    let wine_arc_copy = wine_arc.clone();
                                    let path_arc_copy = path_arc.clone();
                                    let font_queue_copy = font_queue.clone();
                                    let installed_fonts_copy = installed_fonts.clone();

                                    let sender_copy = sender.clone();

                                    threads.push(std::thread::spawn(move || -> anyhow::Result<()> {
                                        // Using "while let" here will lead to the first thread locking the queue
                                        // for it's entire lifetime, making parallelization useless
                                        loop {
                                            let Some(font) = font_queue_copy.lock().unwrap().pop() else {
                                                break;
                                            };
                                            
                                            if !font.is_installed(path_arc_copy.as_ref()) {
                                                wine_arc_copy.as_ref().install_font(font)?;
                                            }

                                            sender_copy.send((
                                                CreatePrefixStatus::InstallingFonts, 
                                                installed_fonts_copy.fetch_add(1, Ordering::Relaxed) + 1, 
                                                total_fonts
                                            ))?;
                                        }

                                        Ok(())
                                    }));
                                }

                                for thread in threads {
                                    thread.join().unwrap()?;
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
