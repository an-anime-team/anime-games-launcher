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

use mlua::prelude::*;

mod game_edition;
mod game_variant;
mod game_launch_info;
mod actions_pipeline;
mod pipeline_action;
mod progress_report;
mod game_settings;

pub use game_edition::*;
pub use game_variant::*;
pub use game_launch_info::*;
pub use actions_pipeline::*;
pub use pipeline_action::*;
pub use progress_report::*;
pub use game_settings::*;

use crate::platform::Platform;

#[derive(Debug, Clone)]
pub struct GameIntegration {
    lua: Lua,

    game_get_editions: Option<LuaFunction>,
    game_get_launch_info: LuaFunction,
    game_get_actions_pipeline: LuaFunction,

    settings_get_property: Option<LuaFunction>,
    settings_set_property: Option<LuaFunction>,
    settings_get_layout: Option<LuaFunction>
}

impl GameIntegration {
    /// Try to load game integration from provided lua engine and integration
    /// table.
    pub fn from_lua(lua: Lua, table: &LuaTable) -> Result<Self, LuaError> {
        if table.get::<u32>("format")? != 1
            && table.get::<u32>("version")? != 1
        {
            return Err(LuaError::external("unsupported game integration format"));
        }

        let game = table.get::<LuaTable>("game")?;
        let settings = table.get::<LuaTable>("settings").ok();

        Ok(Self {
            lua,

            game_get_editions: game.get("get_editions").ok(),
            game_get_launch_info: game.get("get_launch_info")?,
            game_get_actions_pipeline: game.get("get_actions_pipeline")?,

            settings_get_property: settings.as_ref()
                .map(|settings| settings.get("get_property"))
                .transpose()?,

            settings_set_property: settings.as_ref()
                .map(|settings| settings.get("set_property"))
                .transpose()?,

            settings_get_layout: settings.as_ref()
                .map(|settings| settings.get("get_layout"))
                .transpose()?
        })
    }

    /// Try to get list of available game editions.
    ///
    /// Return `Ok(None)` if integration module doesn't provide any editions.
    pub fn get_editions(
        &self,
        platform: &Platform
    ) -> Result<Option<Box<[GameEdition]>>, LuaError> {
        let Some(get_editions) = &self.game_get_editions else {
            return Ok(None);
        };

        get_editions.call::<Option<Vec<LuaTable>>>(platform.to_string())
            .and_then(|editions| {
                editions.map(|editions| {
                    editions.iter()
                        .map(GameEdition::from_lua)
                        .collect::<Result<Box<[_]>, LuaError>>()
                }).transpose()
            })
    }

    /// Try to get params used to launch the game.
    pub fn get_launch_info(
        &self,
        variant: &GameVariant
    ) -> Result<Option<GameLaunchInfo>, LuaError> {
        let variant = variant.to_lua(&self.lua)?;

        self.game_get_launch_info.call::<Option<LuaTable>>(variant)
            .and_then(|pipeline| {
                pipeline.map(|pipeline| {
                    GameLaunchInfo::from_lua(&pipeline)
                }).transpose()
            })
    }

    /// Try to get game actions pipeline.
    pub fn get_actions_pipeline(
        &self,
        variant: &GameVariant
    ) -> Result<Option<ActionsPipeline>, LuaError> {
        let variant = variant.to_lua(&self.lua)?;

        self.game_get_actions_pipeline.call::<Option<LuaTable>>(variant)
            .and_then(|pipeline| {
                pipeline.map(|pipeline| {
                    ActionsPipeline::from_lua(self.lua.clone(), &pipeline)
                }).transpose()
            })
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
        variant: &GameVariant
    ) -> Result<Option<Box<[GameSettingsGroup]>>, LuaError> {
        let Some(get_layout) = &self.settings_get_layout else {
            return Ok(None);
        };

        get_layout.call::<Vec<LuaTable>>(variant.to_lua(&self.lua)?)
            .and_then(|groups| {
                groups.iter()
                    .map(GameSettingsGroup::from_lua)
                    .collect::<Result<Box<[_]>, LuaError>>()
            })
            .map(Some)
    }
}
