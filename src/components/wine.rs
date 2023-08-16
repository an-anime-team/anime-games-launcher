use std::cell::Cell;
use std::path::PathBuf;

use serde_json::Value as Json;

use wincompatlib::wine::ext::WineWithExt;

use wincompatlib::wine::{
    Wine as WincompatlibWine,
    WineArch as WincompatlibWineArch,
    WineLoader as WincompatlibWineLoader
};

use anime_game_core::network::minreq;
use anime_game_core::archive;

use anime_game_core::network::downloader::DownloaderExt;
use anime_game_core::network::downloader::basic::Downloader;

use anime_game_core::updater::UpdaterExt;

use crate::ui::components::game_card::CardVariant;
use crate::ui::components::tasks_queue::{QueuedTask, ResolvedTask};

use crate::{
    config,
    COMPONENTS_FOLDER
};

use crate::components::{
    Updater,
    Status
};

use super::DownloadComponentResolvedTask;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wine {
    pub name: String,
    pub title: String,
    pub uri: String
}

impl Wine {
    /// Resolve component version from the config file
    pub fn from_config() -> anyhow::Result<Self> {
        let components = config::get().components;

        let wine_versions = minreq::get(format!("{}/wine/{}.json", &components.channel, &components.wine.build))
            .send()?
            .json::<Vec<Json>>()?;

        for wine in wine_versions {
            let name = wine.get("name").and_then(Json::as_str);
            let title = wine.get("title").and_then(Json::as_str);
            let uri = wine.get("uri").and_then(Json::as_str);

            if let (Some(name), Some(title), Some(uri)) = (name, title, uri) {
                if name.contains(&components.wine.version) || components.wine.version == "latest" {
                    return Ok(Self {
                        name: name.to_owned(),
                        title: title.to_owned(),
                        uri: uri.to_owned()
                    })
                }
            }
        }

        anyhow::bail!("No appropriate wine version found")
    }

    #[inline]
    /// Get wine component folder path
    pub fn get_folder(&self) -> PathBuf {
        COMPONENTS_FOLDER
            .join("wine")
            .join(&self.name)
    }

    #[inline]
    /// Get path to the wine executable
    pub fn get_executable(&self) -> PathBuf {
        self.get_folder().join("bin/wine64")
    }

    #[inline]
    /// Check if the component is downloaded
    pub fn is_downloaded(&self) -> bool {
        self.get_folder().exists()
    }

    /// Download component
    pub fn download(&self) -> anyhow::Result<Updater> {
        let (sender, receiver) = flume::unbounded();

        let download_uri = self.uri.clone();

        Ok(Updater {
            status: Cell::new(Status::Downloading),
            current: Cell::new(0),
            total: Cell::new(1), // To prevent division by 0

            worker_result: None,
            updater: receiver,

            worker: Some(std::thread::spawn(move || -> anyhow::Result<()> {
                let downloader = Downloader::new(download_uri);

                let path = COMPONENTS_FOLDER.join("wine");
                let archive = path.join(downloader.file_name());

                // Create wine dir if needed

                std::fs::create_dir_all(&path)?;

                // Download update archive

                let mut updater = downloader.download(&archive)?;

                while let Ok(false) = updater.status() {
                    sender.send((
                        Status::Downloading,
                        updater.current(),
                        updater.total()
                    ))?;
                }

                // Extract archive

                let Some(mut updater) = archive::extract(&archive, &path) else {
                    anyhow::bail!("Unable to extract archive: {:?}", archive);
                };

                while let Ok(false) = updater.status() {
                    sender.send((
                        Status::Unpacking,
                        updater.current(),
                        updater.total()
                    ))?;
                }

                std::fs::remove_file(archive)?;

                // Finish downloading

                sender.send((Status::Finished, 0, 1))?;

                Ok(())
            }))
        })
    }

    /// Get wincompatlib descriptor of the current wine version
    /// if it is already installed, and `None` otherwise
    pub fn to_wincompatlib(&self) -> Option<WincompatlibWine> {
        if !self.is_downloaded() {
            return None;
        }

        let wine = WincompatlibWine::from_binary(self.get_executable())
            .with_arch(WincompatlibWineArch::Win64)
            .with_loader(WincompatlibWineLoader::Current);

        Some(wine)
    }
}

#[derive(Debug)]
pub struct DownloadWineQueuedTask {
    pub title: String,
    pub author: String,
    pub version: Wine
}

impl QueuedTask for DownloadWineQueuedTask {
    #[inline]
    fn get_variant(&self) -> CardVariant {
        CardVariant::Component { 
            title: self.title.clone(), 
            author: self.author.clone() 
        }
    }

    #[inline]
    fn get_title(&self) -> &str {
        self.title.as_str()
    }

    #[inline]
    fn get_author(&self) -> &str {
        self.author.as_str()
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        Ok(Box::new(DownloadComponentResolvedTask {
            title: self.title,
            author: self.author,
            updater: self.version.download()?
        }))
    }
}
