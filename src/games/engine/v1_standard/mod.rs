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
pub struct GameIntegration<'lua> {
    lua: &'lua Lua,

    editions: LuaFunction<'lua>,

    game_get_status: LuaFunction<'lua>,
    game_get_diff: LuaFunction<'lua>,
    game_get_launch_info: LuaFunction<'lua>,

    settings_get_property: LuaFunction<'lua>,
    settings_set_property: LuaFunction<'lua>,
    settings_get_layout: LuaFunction<'lua>
}

impl<'lua> GameIntegration<'lua> {
    pub fn from_lua(lua: &'lua Lua, table: &LuaTable<'lua>) -> Result<Self, LuaError> {
        if table.get::<_, u32>("standard")? != 1 {
            return Err(LuaError::external("invalid game integration standard, v1 expected"));
        }

        let game = table.get::<_, LuaTable>("game")?;
        let settings = table.get::<_, LuaTable>("settings")?;

        Ok(Self {
            lua,

            editions: table.get("editions")?,

            game_get_status: game.get("get_status")?,
            game_get_diff: game.get("get_diff")?,
            game_get_launch_info: game.get("get_launch_info")?,

            settings_get_property: settings.get("get_property")?,
            settings_set_property: settings.get("set_property")?,
            settings_get_layout: settings.get("get_layout")?
        })
    }

    #[inline]
    /// Get list of available game editions.
    pub fn editions(&self, platform: TargetPlatform) -> Result<Vec<GameEdition>, LuaError> {
        self.editions.call::<_, Vec<LuaTable>>(platform.to_string())
            .and_then(|editions| {
                editions.iter()
                    .map(GameEdition::try_from)
                    .collect::<Result<Vec<_>, _>>()
            })
    }

    /// Get status of the game installation.
    pub fn game_status(&self, variant: &GameVariant) -> Result<InstallationStatus, LuaError> {
        self.game_get_status.call::<_, LuaString>(variant.to_lua(self.lua)?)
            .and_then(|status| InstallationStatus::from_str(&status.to_string_lossy()))
    }

    /// Get installation diff.
    pub fn game_diff(&self, variant: &GameVariant) -> Result<Option<InstallationDiff>, LuaError> {
        self.game_get_diff.call::<_, Option<LuaTable>>(variant.to_lua(self.lua)?)
            .and_then(|diff| {
                diff.map(|diff| InstallationDiff::from_lua(self.lua, &diff))
                    .transpose()
            })
    }

    /// Get params used to launch the game.
    pub fn game_launch_info(&self, variant: &GameVariant) -> Result<GameLaunchInfo, AsLuaError> {
        self.game_get_launch_info.call::<_, LuaValue>(variant.to_lua(self.lua)?)
            .map_err(AsLuaError::LuaError)
            .and_then(|info| GameLaunchInfo::from_lua(&info))
    }

    /// Get settings param from the game integration module.
    pub fn get_property(&self, name: impl AsRef<str>) -> Result<LuaValue, AsLuaError> {
        self.settings_get_property.call::<_, LuaValue>(name.as_ref())
            .map_err(AsLuaError::LuaError)
    }

    /// Set settings param value.
    pub fn set_property(&self, name: impl AsRef<str>, value: LuaValue) -> Result<(), AsLuaError> {
        self.settings_set_property.call::<_, ()>((name.as_ref(), value))
            .map_err(AsLuaError::LuaError)
    }

    /// Get game settings UI layout.
    pub fn get_settings_layout(&self, variant: &GameVariant) -> Result<Vec<GameSettingsGroup>, AsLuaError> {
        self.settings_get_layout.call::<_, Vec<LuaValue>>(variant.to_lua(self.lua)?)
            .map_err(AsLuaError::LuaError)
            .and_then(|groups| {
                groups.iter()
                    .map(GameSettingsGroup::from_lua)
                    .collect::<Result<Vec<_>, AsLuaError>>()
            })
    }
}
