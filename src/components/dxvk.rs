use std::cell::Cell;
use std::path::PathBuf;

use anime_game_core::network::minreq;
use anime_game_core::archive;

use anime_game_core::network::downloader::DownloaderExt;
use anime_game_core::network::downloader::basic::Downloader;

use anime_game_core::updater::UpdaterExt;

use serde_json::Value as Json;

use crate::{
    config,
    COMPONENTS_FOLDER
};

use crate::components::{
    Updater,
    Status
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dxvk {
    pub name: String,
    pub title: String,
    pub uri: String
}

impl Dxvk {
    /// Resolve component version from the config file
    pub fn from_config() -> anyhow::Result<Self> {
        let components = config::get().components;

        let dxvk_versions = minreq::get(format!("{}/dxvk/{}.json", &components.channel, &components.dxvk.build))
            .send()?
            .json::<Vec<Json>>()?;

        for dxvk in dxvk_versions {
            let name = dxvk.get("name").and_then(Json::as_str);
            let title = dxvk.get("title").and_then(Json::as_str);
            let uri = dxvk.get("uri").and_then(Json::as_str);

            if let (Some(name), Some(title), Some(uri)) = (name, title, uri) {
                if name.contains(&components.dxvk.version) || components.dxvk.version == "latest" {
                    return Ok(Self {
                        name: name.to_owned(),
                        title: title.to_owned(),
                        uri: uri.to_owned()
                    })
                }
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
}
