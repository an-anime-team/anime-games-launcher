use std::path::{Path, PathBuf};

use serde_json::Value as Json;
use mlua::prelude::*;

pub mod manifest;
pub mod standards;

use manifest::Manifest;
use standards::prelude::*;

#[derive(Debug)]
pub struct Game {
    pub manifest: Manifest,
    pub script_path: PathBuf,

    lua: Lua
}

// Let (at least for now) lua scripts maintainers resolve
// possible data races themselves
// 
// FIXME: use Mutex or RwLock, or anything else but please remove this shit

unsafe impl Send for Game {}
unsafe impl Sync for Game {}

impl Game {
    pub fn new(manifest_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let manifest = std::fs::read(manifest_path.as_ref())?;
        let manifest = serde_json::from_slice::<Json>(&manifest)?;
        let manifest = Manifest::from_json(&manifest)?;

        let script_path = PathBuf::from(&manifest.script_path);

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
            manifest,
            script_path,
            lua: Lua::new()
        };

        game.lua.globals().set("v1_network_http_get", game.lua.create_function(|lua, uri: String| {
            anime_game_core::network::minreq::get(uri)
                .send()
                .map(|result| lua.create_string(result.as_bytes()))
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

    pub fn get_card_picture(&self, edition: &str) -> anyhow::Result<String> {
        #[inline]
        #[cached::proc_macro::cached(
            key = "String",
            convert = r##"{ format!("{_game}{edition}") }"##,
            result
        )]
        fn get_card_picture(lua: &Lua, standard: IntegrationStandard, _game: &str, edition: &str) -> anyhow::Result<String> {
            match standard {
                IntegrationStandard::V1 => Ok(lua.globals().call_function("v1_visual_get_card_picture", edition)?)
            }
        }

        get_card_picture(&self.lua, self.manifest.script_standard, &self.manifest.game_name, edition)
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_background_picture(&self, edition: &str) -> anyhow::Result<String> {
        #[inline]
        #[cached::proc_macro::cached(
            key = "String",
            convert = r##"{ format!("{_game}{edition}") }"##,
            result
        )]
        fn get_background_picture(lua: &Lua, standard: IntegrationStandard, _game: &str, edition: &str) -> anyhow::Result<String> {
            match standard {
                IntegrationStandard::V1 => Ok(lua.globals().call_function("v1_visual_get_background_picture", edition)?)
            }
        }

        get_background_picture(&self.lua, self.manifest.script_standard, &self.manifest.game_name, edition)
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_details_background_style(&self, edition: &str) -> anyhow::Result<Option<String>> {
        #[inline]
        #[cached::proc_macro::cached(
            key = "String",
            convert = r##"{ format!("{_game}{edition}") }"##,
            result
        )]
        fn get_details_background_style(lua: &Lua, standard: IntegrationStandard, _game: &str, edition: &str) -> anyhow::Result<Option<String>> {
            match standard {
                IntegrationStandard::V1 => {
                    if !lua.globals().contains_key("v1_visual_get_details_background_css")? {
                        return Ok(None);
                    }

                    Ok(lua.globals().call_function("v1_visual_get_details_background_css", edition)?)
                }
            }
        }

        get_details_background_style(&self.lua, self.manifest.script_standard, &self.manifest.game_name, edition)
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_game_editions_list(&self) -> anyhow::Result<Vec<GameEdition>> {
        #[inline]
        #[cached::proc_macro::cached(
            key = "String",
            convert = r##"{ _game.to_string() }"##,
            result
        )]
        fn get_game_editions_list(lua: &Lua, standard: IntegrationStandard, _game: &str) -> anyhow::Result<Vec<GameEdition>> {
            match standard {
                IntegrationStandard::V1 => {
                    let editions = lua.globals()
                        .call_function::<_, LuaTable>("v1_game_get_editions_list", ())?
                        .sequence_values::<LuaTable>()
                        .flatten()
                        .map(|edition| GameEdition::from_table(edition, standard))
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(editions)
                }
            }
        }

        get_game_editions_list(&self.lua, self.manifest.script_standard, &self.manifest.game_name)
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn is_game_installed(&self, path: &str) -> anyhow::Result<bool> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .call_function("v1_game_is_installed", path)?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_game_version(&self, path: &str, edition: &str) -> anyhow::Result<Option<String>> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .call_function("v1_game_get_version", (path, edition))?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_game_download(&self, edition: &str) -> anyhow::Result<Download> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => {
                let download = self.lua.globals()
                    .call_function("v1_game_get_download", edition)?;

                Download::from_table(download, self.manifest.script_standard)
            }
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_game_diff(&self, path: &str, edition: &str) -> anyhow::Result<Option<Diff>> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => {
                let diff = self.lua.globals()
                    .call_function("v1_game_get_diff", (path, edition))?;

                match diff {
                    Some(diff) => Ok(Some(Diff::from_table(diff, self.manifest.script_standard)?)),
                    None => Ok(None)
                }
            }
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_game_status(&self, path: &str, edition: &str) -> anyhow::Result<Option<GameStatus>> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => {
                let status = self.lua.globals()
                    .call_function("v1_game_get_status", (path, edition))?;

                match status {
                    Some(status) => Ok(Some(GameStatus::from_table(status, self.manifest.script_standard)?)),
                    None => Ok(None)
                }
            }
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_launch_options(&self, game_path: &str, addons_path: &str, edition: &str) -> anyhow::Result<GameLaunchOptions> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => {
                let options = self.lua.globals()
                    .call_function("v1_game_get_launch_options", (game_path, addons_path, edition))?;

                GameLaunchOptions::from_table(options, self.manifest.script_standard)
            }
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn is_process_running(&self, game_path: &str, edition: &str) -> anyhow::Result<bool> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .call_function("v1_games_is_running", (game_path, edition))?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn kill_process(&self, game_path: &str, edition: &str) -> anyhow::Result<()> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .call_function("v1_game_kill", (game_path, edition))?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_game_integrity(&self, game_path: &str, edition: &str) -> anyhow::Result<Vec<IntegrityInfo>> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => {
                let info = self.lua.globals()
                    .call_function::<_, LuaTable>("v1_game_get_integrity_info", (game_path, edition))?
                    .sequence_values::<LuaTable>()
                    .flatten()
                    .map(|info| IntegrityInfo::from_table(info, self.manifest.script_standard))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(info)
            }
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_addons_list(&self, edition: &str) -> anyhow::Result<Vec<AddonsGroup>> {
        #[inline]
        #[cached::proc_macro::cached(
            key = "String",
            convert = r##"{ format!("{_game}{edition}") }"##,
            result
        )]
        fn get_addons_list(lua: &Lua, standard: IntegrationStandard, _game: &str, edition: &str) -> anyhow::Result<Vec<AddonsGroup>> {
            match standard {
                IntegrationStandard::V1 => {
                    let addons = lua.globals()
                        .call_function::<_, LuaTable>("v1_addons_get_list", edition)?
                        .sequence_values::<LuaTable>()
                        .flatten()
                        .map(|group| AddonsGroup::from_table(group, standard))
                        .collect::<Result<Vec<_>, _>>()?;

                    Ok(addons)
                }
            }
        }

        get_addons_list(&self.lua, self.manifest.script_standard, &self.manifest.game_name, edition)
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn is_addon_installed(&self, group_name: &str, addon_name: &str, addon_path: &str, edition: &str) -> anyhow::Result<bool> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .call_function("v1_addons_is_installed", (
                    group_name,
                    addon_name,
                    addon_path,
                    edition
                ))?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_addon_version(&self, group_name: &str, addon_name: &str, addon_path: &str, edition: &str) -> anyhow::Result<Option<String>> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .call_function("v1_addons_get_version", (
                    group_name,
                    addon_name,
                    addon_path,
                    edition
                ))?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_addon_download(&self, group_name: &str, addon_name: &str, edition: &str) -> anyhow::Result<Download> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => {
                let download = self.lua.globals()
                    .call_function("v1_addons_get_download", (
                        group_name,
                        addon_name,
                        edition
                    ))?;

                Download::from_table(download, self.manifest.script_standard)
            }
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_addon_diff(&self, group_name: &str, addon_name: &str, addon_path: &str, edition: &str) -> anyhow::Result<Option<Diff>> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => {
                let diff = self.lua.globals()
                    .call_function("v1_addons_get_diff", (
                        group_name,
                        addon_name,
                        addon_path,
                        edition
                    ))?;

                match diff {
                    Some(diff) => Ok(Some(Diff::from_table(diff, self.manifest.script_standard)?)),
                    None => Ok(None)
                }
            }
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_addon_paths(&self, group_name: &str, addon_name: &str, addon_path: &str, edition: &str) -> anyhow::Result<Vec<String>> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .call_function("v1_addons_get_paths", (
                    group_name,
                    addon_name,
                    addon_path,
                    edition
                ))?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn get_addon_integrity(&self, group_name: &str, addon_name: &str, addon_path: &str, edition: &str) -> anyhow::Result<Vec<IntegrityInfo>> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => {
                let info = self.lua.globals()
                    .call_function::<_, LuaTable>("v1_addons_get_integrity_info", (
                        group_name,
                        addon_name,
                        addon_path,
                        edition
                    ))?
                    .sequence_values::<LuaTable>()
                    .flatten()
                    .map(|info| IntegrityInfo::from_table(info, self.manifest.script_standard))
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(info)
            }
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn has_game_diff_transition(&self) -> anyhow::Result<bool> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals().contains_key("v1_game_diff_transition")?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn run_game_diff_transition(&self, transition_path: &str, edition: &str) -> anyhow::Result<()> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .call_function("v1_game_diff_transition", (transition_path, edition))?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn has_game_diff_post_transition(&self) -> anyhow::Result<bool> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals().contains_key("v1_game_diff_post_transition")?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn run_game_diff_post_transition(&self, path: &str, edition: &str) -> anyhow::Result<()> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .call_function("v1_game_diff_post_transition", (path, edition))?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn has_addons_diff_transition(&self) -> anyhow::Result<bool> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals().contains_key("v1_addons_diff_transition")?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn run_addons_diff_transition(&self, group_name: &str, addon_name: &str, transition_path: &str, edition: &str) -> anyhow::Result<()> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .call_function("v1_addons_diff_transition", (
                    group_name,
                    addon_name,
                    transition_path,
                    edition
                ))?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn has_addons_diff_post_transition(&self) -> anyhow::Result<bool> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals().contains_key("v1_addons_diff_post_transition")?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn run_addons_diff_post_transition(&self, group_name: &str, addon_name: &str, addon_path: &str, edition: &str) -> anyhow::Result<()> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .call_function("v1_addons_diff_post_transition", (
                    group_name,
                    addon_name,
                    addon_path,
                    edition
                ))?)
        }
    }

    #[tracing::instrument(level = "trace", ret)]
    pub fn has_integrity_hash(&self) -> anyhow::Result<bool> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals().contains_key("v1_integrity_hash")?)
        }
    }

    #[tracing::instrument(level = "trace", skip(data), ret)]
    pub fn integrity_hash(&self, algorithm: &str, data: impl AsRef<[u8]>) -> anyhow::Result<String> {
        match self.manifest.script_standard {
            IntegrationStandard::V1 => Ok(self.lua.globals()
                .call_function("v1_integrity_hash", (
                    algorithm,
                    self.lua.create_string(data)?
                ))?)
        }
    }
}
