use std::cell::Cell;
use std::path::PathBuf;

use serde_json::Value as Json;

use anime_game_core::network::minreq;
use anime_game_core::archive;

use anime_game_core::network::downloader::DownloaderExt;
use anime_game_core::network::downloader::basic::Downloader;

use anime_game_core::updater::UpdaterExt;

use crate::{
    config,
    COMPONENTS_FOLDER
};

use crate::components::{
    Updater,
    Status
};

use crate::ui::components::game_card::CardInfo;
use crate::ui::components::tasks_queue::{QueuedTask, ResolvedTask};

use super::DownloadComponentResolvedTask;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dxvk {
    pub name: String,
    pub title: String,
    pub uri: String
}

impl Dxvk {
    /// Get selected wine build versions list
    pub fn versions() -> anyhow::Result<Vec<Self>> {
        let components = config::get().components;

        let dxvk_versions = minreq::get(format!("{}/dxvk/{}.json", &components.channel, &components.dxvk.build))
            .send()?.json::<Vec<Json>>()?;

        let mut versions = Vec::new();

        for dxvk in dxvk_versions {
            let name = dxvk.get("name").and_then(Json::as_str);
            let title = dxvk.get("title").and_then(Json::as_str);
            let uri = dxvk.get("uri").and_then(Json::as_str);

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
        let dxvk_info = config::get().components.dxvk;

        for version in Self::versions()? {
            if version.name.contains(&dxvk_info.version) || dxvk_info.version == "latest" {
                return Ok(version);
            }
        }

        anyhow::bail!("No appropriate dxvk version found")
    }

    #[inline]
    /// Get dxvk component folder path
    pub fn get_folder(&self) -> PathBuf {
        COMPONENTS_FOLDER
            .join("dxvk")
            .join(&self.name)
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

                let path = COMPONENTS_FOLDER.join("dxvk");
                let archive = path.join(downloader.file_name());

                // Create dxvk dir if needed

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

                while let Ok(false) = updater.status() {
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
}

#[derive(Debug)]
pub struct DownloadDxvkQueuedTask {
    pub card_info: CardInfo,
    pub version: Dxvk
}

impl QueuedTask for DownloadDxvkQueuedTask {
    #[inline]
    fn get_info(&self) -> CardInfo {
        self.card_info.clone()
    }

    fn resolve(self: Box<Self>) -> anyhow::Result<Box<dyn ResolvedTask>> {
        Ok(Box::new(DownloadComponentResolvedTask {
            card_info: self.card_info,
            updater: self.version.download()?
        }))
    }
}
