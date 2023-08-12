use anime_game_core::network::minreq;
use anime_game_core::updater::UpdaterExt;

use serde_json::Value as Json;

use crate::config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wine {
    pub name: String,
    pub title: String,
    pub uri: String
}

impl Wine {
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
}

pub struct Updater {

}

impl UpdaterExt for Updater {
    type Status = ();
    type Error = ();
    type Result = ();

    fn status(&mut self) -> Result<Self::Status, &Self::Error> {
        todo!()
    }

    fn wait(self) -> Result<Self::Result, Self::Error> {
        todo!()
    }

    fn is_finished(&mut self) -> bool {
        todo!()
    }

    fn current(&self) -> usize {
        todo!()
    }

    fn total(&self) -> usize {
        todo!()
    }
}
