// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::str::FromStr;

use mlua::prelude::*;

mod game_edition;
mod game_variant;
mod game_launch_info;
mod installation_status;
mod installation_diff;
mod pipeline_action;
mod progress_report;
mod game_settings;

pub use game_edition::*;
pub use game_variant::*;
pub use game_launch_info::*;
pub use installation_status::*;
pub use installation_diff::*;
pub use pipeline_action::*;
pub use progress_report::*;
pub use game_settings::*;

use crate::platform::Platform;

#[derive(Debug, Clone)]
pub struct GameIntegration {
    lua: Lua,

    game_get_editions: Option<LuaFunction>,
    game_get_status: LuaFunction,
    game_get_diff: LuaFunction,
    game_get_launch_info: LuaFunction,

    settings_get_property: Option<LuaFunction>,
    settings_set_property: Option<LuaFunction>,
    settings_get_layout: Option<LuaFunction>
}

impl GameIntegration {
    /// Try to load game integration from provided lua engine and integration
    /// table.
    pub fn from_lua(lua: Lua, table: &LuaTable) -> Result<Self, LuaError> {
        if table.get::<u32>("version")? != 1 {
            return Err(LuaError::external("unsupported game integration version"));
        }

        let game = table.get::<LuaTable>("game")?;
        let settings = table.get::<LuaTable>("settings").ok();

        Ok(Self {
            lua,

            game_get_editions: game.get("get_editions").ok(),
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
    /// Return `Ok(None)` if integration module doesn't provide any editions.
    pub fn game_editions(
        &self,
        platform: impl AsRef<Platform>
    ) -> Result<Option<Box<[GameEdition]>>, LuaError> {
        let Some(get_editions) = &self.game_get_editions else {
            return Ok(None);
        };

        get_editions.call::<Option<Vec<LuaTable>>>(platform.as_ref().to_string())
            .and_then(|editions| {
                editions.map(|editions| {
                    editions.iter()
                        .map(GameEdition::from_lua)
                        .collect::<Result<Box<[_]>, LuaError>>()
                }).transpose()
            })
    }

    /// Get status of the game installation.
    pub fn game_status(
        &self,
        variant: impl AsRef<GameVariant>
    ) -> Result<InstallationStatus, LuaError> {
        let variant = variant.as_ref()
            .to_lua(&self.lua)?;

        self.game_get_status.call::<String>(variant)
            .and_then(|status| {
                InstallationStatus::from_str(&status)
                    .map_err(|_| LuaError::external("invalid game installation status"))
            })
    }

    /// Get installation diff.
    pub fn game_diff(
        &self,
        variant: impl AsRef<GameVariant>
    ) -> Result<Option<InstallationDiff>, LuaError> {
        let variant = variant.as_ref()
            .to_lua(&self.lua)?;

        self.game_get_diff.call::<Option<LuaTable>>(variant)
            .and_then(|diff| {
                diff.map(|diff| {
                    InstallationDiff::from_lua(self.lua.clone(), &diff)
                }).transpose()
            })
    }

    /// Get params used to launch the game.
    pub fn game_launch_info(
        &self,
        variant: impl AsRef<GameVariant>
    ) -> Result<GameLaunchInfo, LuaError> {
        let variant = variant.as_ref()
            .to_lua(&self.lua)?;

        self.game_get_launch_info.call::<LuaTable>(variant)
            .and_then(|info| GameLaunchInfo::from_lua(&info))
    }

    /// Get settings param from the game integration module.
    ///
    /// Return `Ok(None)` if settings are not specified. Otherwise return the
    /// read property value.
    pub fn get_property(
        &self,
        name: impl AsRef<str>
    ) -> Result<Option<LuaValue>, LuaError> {
        let Some(get_property) = &self.settings_get_property else {
            return Ok(None);
        };

        get_property.call::<LuaValue>(name.as_ref())
            .map(Some)
    }

    /// Set settings param value to the game integration module. Do nothing if
    /// settings are not specified.
    pub fn set_property(
        &self,
        name: impl AsRef<str>,
        value: impl IntoLua
    ) -> Result<(), LuaError> {
        let Some(set_property) = &self.settings_set_property else {
            return Ok(());
        };

        set_property.call::<()>((name.as_ref(), value.into_lua(&self.lua)?))
    }

    /// Get game settings layout.
    ///
    /// Return `Ok(None)` if settings are not specified.
    pub fn get_settings_layout(
        &self,
        variant: impl AsRef<GameVariant>
    ) -> Result<Option<Box<[GameSettingsGroup]>>, LuaError> {
        let Some(get_layout) = &self.settings_get_layout else {
            return Ok(None);
        };

        get_layout.call::<Vec<LuaTable>>(variant.as_ref().to_lua(&self.lua)?)
            .and_then(|groups| {
                groups.iter()
                    .map(GameSettingsGroup::from_lua)
                    .collect::<Result<Box<[_]>, LuaError>>()
            })
            .map(Some)
    }
}
