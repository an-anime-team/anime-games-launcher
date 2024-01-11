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

use crate::ui::components::game_card::CardInfo;
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
    /// Get selected wine build versions list
    pub fn versions() -> anyhow::Result<Vec<Self>> {
        let components = config::get().components;

        let wine_versions = minreq::get(format!("{}/wine/{}.json", &components.channel, &components.wine.build))
            .send()?.json::<Vec<Json>>()?;

        let mut versions = Vec::new();

        for wine in wine_versions {
            let name = wine.get("name").and_then(Json::as_str);
            let title = wine.get("title").and_then(Json::as_str);
            let uri = wine.get("uri").and_then(Json::as_str);

            if let (Some(name), Some(title), Some(uri)) = (name, title, uri) {
                versions.push(Self {
                    name: name.to_owned(),
                    title: title.to_owned(),
                    uri: uri.to_owned()
                });
            }
        }

        Ok(versions)
    }

    /// Resolve component version from the config file
    pub fn from_config() -> anyhow::Result<Self> {
        let wine_info = config::get().components.wine;

        for version in Self::versions()? {
            if version.name.contains(&wine_info.version) || wine_info.version == "latest" {
                return Ok(version);
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

                while !updater.is_finished() {
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

                while !updater.is_finished() {
                    sender.send((
                        Status::Unpacking,
                        updater.current(),
                        updater.total()
                    ))?;
                }

                std::fs::remove_file(archive)?;

                // Finish downloading

                sender.send((Status::Finished, 1, 1))?;

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
    pub name: String,
    pub title: String,
    pub developer: String,
    pub version: Wine
}

impl QueuedTask for DownloadWineQueuedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        CardInfo::Component {
            name: self.name.clone(),
            title: self.title.clone(),
            developer: self.developer.clone()
        }
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        Ok(Box::new(DownloadComponentResolvedTask {
            name: self.name,
            title: self.title,
            developer: self.developer,
            updater: self.version.download()?
        }))
    }
}
