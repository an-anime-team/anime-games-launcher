use std::path::{Path, PathBuf};

use serde_json::Value as Json;
use mlua::prelude::*;

pub mod standards;

use standards::prelude::*;

#[derive(Debug)]
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
    pub fn new(manifest_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let manifest = std::fs::read(manifest_path.as_ref())?;
        let manifest = serde_json::from_slice::<Json>(&manifest)?;

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

                let script = std::fs::read_to_string(&script_path)?;

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

                game.lua.globals().set("v1_json_decode", game.lua.create_function(|lua, json: String| {
                    serde_json::from_str::<Json>(&json)
                        .map(|value| lua.to_value(&value))
                        .map_err(LuaError::external)
                })?)?;

                game.lua.load(script).exec()?;

                Ok(game)
            }

            Some(version) => anyhow::bail!("Unknown manifest version: {version}"),
            None => anyhow::bail!("Wrong manifest file structure")
        }
    }

    pub fn get_card_picture(&self, edition: impl AsRef<str>) -> anyhow::Result<String> {
        match self.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .get::<_, LuaFunction>("v1_visual_get_card_picture")?
                .call::<_, String>(edition.as_ref())?)
        }
    }

    pub fn get_background_picture(&self, edition: impl AsRef<str>) -> anyhow::Result<String> {
        match self.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .get::<_, LuaFunction>("v1_visual_get_background_picture")?
                .call::<_, String>(edition.as_ref())?)
        }
    }

    pub fn get_game_editions_list(&self) -> anyhow::Result<Vec<GameEdition>> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let editions = self.lua.globals()
                    .get::<_, LuaFunction>("v1_game_get_editions_list")?
                    .call::<_, LuaTable>(())?
                    .sequence_values::<LuaTable>()
                    .flatten()
                    .flat_map(|edition| GameEdition::from_table(edition, self.script_standard))
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

    pub fn get_game_version(&self, path: impl AsRef<str>, edition: impl AsRef<str>) -> anyhow::Result<Option<String>> {
        match self.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .get::<_, LuaFunction>("v1_game_get_version")?
                .call::<_, Option<String>>((path.as_ref(), edition.as_ref()))?)
        }
    }

    pub fn get_game_download(&self, edition: impl AsRef<str>) -> anyhow::Result<Download> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let download = self.lua.globals()
                    .get::<_, LuaFunction>("v1_game_get_download")?
                    .call::<_, LuaTable>(edition.as_ref())?;

                Download::from_table(download, self.script_standard)
            }
        }
    }

    pub fn get_game_diff(&self, path: impl AsRef<str>, edition: impl AsRef<str>) -> anyhow::Result<Option<Diff>> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let diff = self.lua.globals()
                    .get::<_, LuaFunction>("v1_game_get_diff")?
                    .call::<_, Option<LuaTable>>((path.as_ref(), edition.as_ref()))?;

                match diff {
                    Some(diff) => Ok(Some(Diff::from_table(diff, self.script_standard)?)),
                    None => Ok(None)
                }
            }
        }
    }

    pub fn get_game_status(&self, path: impl AsRef<str>, edition: impl AsRef<str>) -> anyhow::Result<Option<GameStatus>> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let status = self.lua.globals()
                    .get::<_, LuaFunction>("v1_game_get_status")?
                    .call::<_, Option<LuaTable>>((path.as_ref(), edition.as_ref()))?;

                match status {
                    Some(status) => Ok(Some(GameStatus::from_table(status, self.script_standard)?)),
                    None => Ok(None)
                }
            }
        }
    }

    pub fn get_launch_options(&self, game_path: impl AsRef<str>, addons_path: impl AsRef<str>, edition: impl AsRef<str>) -> anyhow::Result<GameLaunchOptions> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let options = self.lua.globals()
                    .get::<_, LuaFunction>("v1_game_get_launch_options")?
                    .call::<_, LuaTable>((game_path.as_ref(), addons_path.as_ref(), edition.as_ref()))?;

                GameLaunchOptions::from_table(options, self.script_standard)
            }
        }
    }

    pub fn get_addons_list(&self, edition: impl AsRef<str>) -> anyhow::Result<Vec<AddonsGroup>> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let dlcs = self.lua.globals()
                    .get::<_, LuaFunction>("v1_addons_get_list")?
                    .call::<_, LuaTable>(edition.as_ref())?
                    .sequence_values::<LuaTable>()
                    .flatten()
                    .flat_map(|group| AddonsGroup::from_table(group, self.script_standard))
                    .collect();

                Ok(dlcs)
            }
        }
    }

    pub fn is_addon_installed(&self, group_name: impl AsRef<str>, addon_name: impl AsRef<str>, addon_path: impl AsRef<str>, edition: impl AsRef<str>) -> anyhow::Result<bool> {
        match self.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .get::<_, LuaFunction>("v1_addons_is_installed")?
                .call::<_, bool>((
                    group_name.as_ref(),
                    addon_name.as_ref(),
                    addon_path.as_ref(),
                    edition.as_ref()
                ))?)
        }
    }

    pub fn get_addon_version(&self, group_name: impl AsRef<str>, addon_name: impl AsRef<str>, addon_path: impl AsRef<str>, edition: impl AsRef<str>) -> anyhow::Result<Option<String>> {
        match self.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .get::<_, LuaFunction>("v1_addons_get_version")?
                .call::<_, Option<String>>((
                    group_name.as_ref(),
                    addon_name.as_ref(),
                    addon_path.as_ref(),
                    edition.as_ref()
                ))?)
        }
    }

    pub fn get_addon_download(&self, group_name: impl AsRef<str>, addon_name: impl AsRef<str>, edition: impl AsRef<str>) -> anyhow::Result<Download> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let download = self.lua.globals()
                    .get::<_, LuaFunction>("v1_addons_get_download")?
                    .call::<_, LuaTable>((
                        group_name.as_ref(),
                        addon_name.as_ref(),
                        edition.as_ref()
                    ))?;

                Download::from_table(download, self.script_standard)
            }
        }
    }

    pub fn get_addon_diff(&self, group_name: impl AsRef<str>, addon_name: impl AsRef<str>, addon_path: impl AsRef<str>, edition: impl AsRef<str>) -> anyhow::Result<Diff> {
        match self.script_standard {
            IntegrationStandard::V1 => {
                let diff = self.lua.globals()
                    .get::<_, LuaFunction>("v1_addons_get_diff")?
                    .call::<_, LuaTable>((
                        group_name.as_ref(),
                        addon_name.as_ref(),
                        addon_path.as_ref(),
                        edition.as_ref()
                    ))?;

                Diff::from_table(diff, self.script_standard)
            }
        }
    }

    pub fn has_diff_transition(&self) -> anyhow::Result<bool> {
        match self.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals().contains_key("v1_diff_transition")?)
        }
    }

    pub fn run_diff_transition(&self, transition_path: impl AsRef<str>, edition: impl AsRef<str>) -> anyhow::Result<()> {
        match self.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .get::<_, LuaFunction>("v1_diff_transition")?
                .call::<_, ()>((transition_path.as_ref(), edition.as_ref()))?)
        }
    }

    pub fn has_diff_post_transition(&self) -> anyhow::Result<bool> {
        match self.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals().contains_key("v1_diff_post_transition")?)
        }
    }

    pub fn run_diff_post_transition(&self, path: impl AsRef<str>, edition: impl AsRef<str>) -> anyhow::Result<()> {
        match self.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .get::<_, LuaFunction>("v1_diff_post_transition")?
                .call::<_, ()>((path.as_ref(), edition.as_ref()))?)
        }
    }
}
