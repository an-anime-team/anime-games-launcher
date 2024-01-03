use std::path::{Path, PathBuf};

use serde_json::Value as Json;
use mlua::prelude::*;

use anime_game_core::filesystem::DriverExt;

pub mod standards;

use standards::IntegrationStandard;

pub struct Game {
    pub game_name: String,
    pub game_title: String,
    pub game_developer: String,

    pub script_path: PathBuf,
    pub script_version: String,
    pub script_standard: IntegrationStandard,

    lua: Lua
}

impl Game {
    pub fn new(driver: &impl DriverExt, manifest_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let manifest = serde_json::from_slice::<Json>(&driver.read(manifest_path.as_ref().as_os_str())?)?;

        match manifest.get("manifest_version").and_then(Json::as_str) {
            Some("1") => {
                let Some(game_manifest) = manifest.get("game") else {
                    anyhow::bail!("Wrong manifest v1 structure: field `game` expected but wasn't presented");
                };

                let Some(script_manifest) = manifest.get("script") else {
                    anyhow::bail!("Wrong manifest v1 structure: field `script` expected but wasn't presented");
                };

                let Some(script_path) = script_manifest.get("path").and_then(Json::as_str) else {
                    anyhow::bail!("Wrong manifest v1 structure: field `script.path` expected but wasn't presented");
                };

                let script_path = PathBuf::from(script_path);

                let script_path = if script_path.is_absolute() {
                    script_path
                } else {
                    manifest_path.as_ref()
                        .parent()
                        .map(|path| path.join(&script_path))
                        .unwrap_or(script_path)
                };

                let script = driver.read_to_string(script_path.as_os_str())?;

                let game = Self {
                    game_name: match game_manifest.get("name").and_then(Json::as_str) {
                        Some(name) => name.to_string(),
                        None => anyhow::bail!("Wrong manifest v1 structure: field `game.name` expected but wasn't presented")
                    },

                    game_title: match game_manifest.get("title").and_then(Json::as_str) {
                        Some(title) => title.to_string(),
                        None => anyhow::bail!("Wrong manifest v1 structure: field `game.title` expected but wasn't presented")
                    },

                    game_developer: match game_manifest.get("developer").and_then(Json::as_str) {
                        Some(developer) => developer.to_string(),
                        None => anyhow::bail!("Wrong manifest v1 structure: field `game.developer` expected but wasn't presented")
                    },

                    script_version: match script_manifest.get("version").and_then(Json::as_str) {
                        Some(version) => version.to_string(),
                        None => anyhow::bail!("Wrong manifest v1 structure: field `script.version` expected but wasn't presented")
                    },

                    script_standard: match script_manifest.get("standard").and_then(Json::as_str) {
                        Some("1") => IntegrationStandard::V1,

                        Some(version) => anyhow::bail!("Wrong manifest v1 structure: field `script.standard` containts unknown version: {version}"),
                        None => anyhow::bail!("Wrong manifest v1 structure: field `script.standard` expected but wasn't presented")
                    },

                    script_path,

                    lua: Lua::new()
                };

                game.lua.globals().set("v1_network_http_get", game.lua.create_function(|_, uri: String| {
                    anime_game_core::network::minreq::get(uri)
                        .send()
                        .and_then(|result| result.as_str().map(String::from))
                        .map_err(LuaError::external)
                })?)?;

                // game.lua.globals().set("json_decode", game.lua.create_function(|_, json: String| {
                //     serde_json::from_str::<serde_json::Value>(&json)
                //         .and_then(|value| value.)
                //         .map_err(LuaError::external)
                // })?)?;

                game.lua.load(script).exec()?;

                Ok(game)
            }

            Some(version) => anyhow::bail!("Unknown manifest version: {version}"),
            None => anyhow::bail!("Wrong manifest file structure")
        }
    }

    pub fn get_card_picture(&self) -> anyhow::Result<String> {
        match self.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .get::<_, LuaFunction>("v1_visual_get_card_picture")?
                .call::<_, String>(())?)
        }
    }

    pub fn get_background_picture(&self) -> anyhow::Result<String> {
        match self.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .get::<_, LuaFunction>("v1_visual_get_background_picture")?
                .call::<_, String>(())?)
        }
    }

    pub fn get_game_editions_list(&self) -> anyhow::Result<Vec<standards::game::Edition>> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let editions = self.lua.globals()
                    .get::<_, LuaFunction>("v1_game_get_editions_list")?
                    .call::<_, LuaTable>(())?
                    .sequence_values::<LuaTable>()
                    .flatten()
                    .flat_map(|edition| standards::game::Edition::from_table(edition, self.script_standard))
                    .collect();

                Ok(editions)
            }
        }
    }

    pub fn is_game_installed(&self, path: impl AsRef<str>) -> anyhow::Result<bool> {
        match self.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .get::<_, LuaFunction>("v1_game_is_installed")?
                .call::<_, bool>(path.as_ref())?)
        }
    }

    pub fn get_game_info(&self, path: impl AsRef<str>) -> anyhow::Result<Option<standards::game::GameInfo>> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let info = self.lua.globals()
                    .get::<_, LuaFunction>("v1_game_get_info")?
                    .call::<_, Option<LuaTable>>(path.as_ref())?;

                match info {
                    Some(info) => Ok(Some(standards::game::GameInfo::from_table(info, self.script_standard)?)),
                    None => Ok(None)
                }
            }
        }
    }

    pub fn get_game_download(&self, edition: impl AsRef<str>) -> anyhow::Result<standards::game::Download> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let download = self.lua.globals()
                    .get::<_, LuaFunction>("v1_game_get_download")?
                    .call::<_, LuaTable>(edition.as_ref())?;

                standards::game::Download::from_table(download, self.script_standard)
            }
        }
    }

    pub fn get_game_diff(&self, path: impl AsRef<str>) -> anyhow::Result<Option<standards::game::Diff>> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let diff = self.lua.globals()
                    .get::<_, LuaFunction>("v1_game_get_diff")?
                    .call::<_, Option<LuaTable>>(path.as_ref())?;

                match diff {
                    Some(diff) => Ok(Some(standards::game::Diff::from_table(diff, self.script_standard)?)),
                    None => Ok(None)
                }
            }
        }
    }

    pub fn get_dlc_list(&self, edition: impl AsRef<str>) -> anyhow::Result<Vec<standards::dlc::Group>> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let dlcs = self.lua.globals()
                    .get::<_, LuaFunction>("v1_dlc_get_list")?
                    .call::<_, LuaTable>(edition.as_ref())?
                    .sequence_values::<LuaTable>()
                    .flatten()
                    .flat_map(|group| standards::dlc::Group::from_table(group, self.script_standard))
                    .collect();

                Ok(dlcs)
            }
        }
    }

    pub fn create_task_from_game_diff(&self, diff: &standards::game::Diff) -> anyhow::Result<Option<u64>> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let task_id = self.lua.globals()
                    .get::<_, LuaFunction>("v1_tasks_create_from_game_diff")?
                    .call::<_, Option<u64>>(diff.to_table(&self.lua, self.script_standard)?)?;

                Ok(task_id)
            }
        }
    }

    pub fn get_task_status(&self, task_id: u64) -> anyhow::Result<Option<standards::tasks::Info>> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let task_status = self.lua.globals()
                    .get::<_, LuaFunction>("v1_tasks_get_status")?
                    .call::<_, Option<LuaTable>>(task_id)?;

                let Some(task_status) = task_status else {
                    return Ok(None);
                };

                Ok(Some(standards::tasks::Info::from_table(task_status, self.script_standard)?))
            }
        }
    }
}
