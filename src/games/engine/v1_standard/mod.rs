use std::str::FromStr;

use mlua::prelude::*;

use crate::prelude::*;

mod game_edition;
mod game_variant;
mod game_launch_status;
mod game_launch_info;
mod installation_status;
mod installation_diff;
mod pipeline_action;
mod progress_report;
mod game_settings;

pub use game_edition::*;
pub use game_variant::*;
pub use game_launch_status::*;
pub use game_launch_info::*;
pub use installation_status::*;
pub use installation_diff::*;
pub use pipeline_action::*;
pub use progress_report::*;
pub use game_settings::*;

#[derive(Debug, Clone)]
pub struct GameIntegration {
    lua: Lua,

    editions: Option<LuaFunction>,

    game_get_status: LuaFunction,
    game_get_diff: LuaFunction,
    game_get_launch_info: LuaFunction,

    settings_get_property: Option<LuaFunction>,
    settings_set_property: Option<LuaFunction>,
    settings_get_layout: Option<LuaFunction>
}

impl GameIntegration {
    pub fn from_lua(lua: Lua, table: &LuaTable) -> Result<Self, LuaError> {
        if table.get::<u32>("standard")? != 1 {
            return Err(LuaError::external("invalid game integration standard, v1 expected"));
        }

        let game = table.get::<LuaTable>("game")?;
        let settings = table.get::<LuaTable>("settings").ok();

        Ok(Self {
            lua,

            editions: table.get("editions").ok(),

            game_get_status: game.get("get_status")?,
            game_get_diff: game.get("get_diff")?,
            game_get_launch_info: game.get("get_launch_info")?,

            settings_get_property: settings.as_ref().map(|settings| settings.get("get_property")).transpose()?,
            settings_set_property: settings.as_ref().map(|settings| settings.get("set_property")).transpose()?,
            settings_get_layout: settings.as_ref().map(|settings| settings.get("get_layout")).transpose()?
        })
    }

    /// Get list of available game editions.
    ///
    /// Return `None` if integration module doesn't provide any editions.
    pub fn editions(&self, platform: TargetPlatform) -> Result<Option<Vec<GameEdition>>, LuaError> {
        match &self.editions {
            Some(editions) => editions.call::<Option<Vec<LuaTable>>>(platform.to_string())
                .and_then(|editions| {
                    editions.map(|editions| {
                        editions.iter()
                            .map(GameEdition::try_from)
                            .collect::<Result<Vec<_>, _>>()
                    }).transpose()
                }),

            None => Ok(None)
        }
    }

    /// Get status of the game installation.
    pub fn game_status(&self, variant: impl AsRef<GameVariant>) -> Result<InstallationStatus, LuaError> {
        self.game_get_status.call::<LuaString>(variant.as_ref().to_lua(&self.lua)?)
            .and_then(|status| InstallationStatus::from_str(&status.to_string_lossy()))
    }

    /// Get installation diff.
    pub fn game_diff(&self, variant: impl AsRef<GameVariant>) -> Result<Option<InstallationDiff>, LuaError> {
        self.game_get_diff.call::<Option<LuaTable>>(variant.as_ref().to_lua(&self.lua)?)
            .and_then(|diff| {
                diff.map(|diff| InstallationDiff::from_lua(self.lua.clone(), &diff))
                    .transpose()
            })
    }

    /// Get params used to launch the game.
    pub fn game_launch_info(&self, variant: impl AsRef<GameVariant>) -> Result<GameLaunchInfo, AsLuaError> {
        self.game_get_launch_info.call::<LuaValue>(variant.as_ref().to_lua(&self.lua)?)
            .map_err(AsLuaError::LuaError)
            .and_then(|info| GameLaunchInfo::from_lua(&info))
    }

    /// Get settings param from the game integration module.
    ///
    /// Return `Ok(None)` if settings are not specified.
    pub fn get_property(&self, name: impl AsRef<str>) -> Result<Option<LuaValue>, AsLuaError> {
        match &self.settings_get_property {
            Some(get_property) => get_property.call::<LuaValue>(name.as_ref())
                .map(Some)
                .map_err(AsLuaError::LuaError),

            None => Ok(None)
        }
    }

    /// Set settings param value.
    ///
    /// Do nothing if settings are not specified.
    pub fn set_property(&self, name: impl AsRef<str>, value: LuaValue) -> Result<(), AsLuaError> {
        match &self.settings_set_property {
            Some(set_property) => set_property.call::<()>((name.as_ref(), value))
                .map_err(AsLuaError::LuaError),

            None => Ok(())
        }
    }

    /// Get game settings UI layout.
    ///
    /// Return `Ok(None)` if settings are not specified.
    pub fn get_settings_layout(&self, variant: impl AsRef<GameVariant>) -> Result<Option<Vec<GameSettingsGroup>>, AsLuaError> {
        match &self.settings_get_layout {
            Some(get_layout) => get_layout.call::<Vec<LuaValue>>(variant.as_ref().to_lua(&self.lua)?)
                .map_err(AsLuaError::LuaError)
                .and_then(|groups| {
                    groups.iter()
                        .map(GameSettingsGroup::from_lua)
                        .collect::<Result<Vec<_>, AsLuaError>>()
                })
                .map(Some),

            None => Ok(None)
        }
    }
}
