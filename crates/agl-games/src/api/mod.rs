// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
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
mod game_components;
mod tools_buttons;
mod game_settings;

pub use game_edition::*;
pub use game_variant::*;
pub use game_launch_info::*;
pub use actions_pipeline::*;
pub use pipeline_action::*;
pub use progress_report::*;
pub use game_components::*;
pub use tools_buttons::*;
pub use game_settings::*;

use crate::platform::Platform;

#[derive(Debug, Clone)]
pub struct GameIntegrationOptions {
    pub platform: Platform
}

#[derive(Debug, Clone)]
pub struct GameIntegration {
    lua: Lua,

    game_get_editions: Option<LuaFunction>,
    game_get_launch_info: LuaFunction,
    game_get_actions_pipeline: LuaFunction,

    components_get_layout: Option<LuaFunction>,
    components_get_enabled: Option<LuaFunction>,
    components_set_enabled: Option<LuaFunction>,
    components_install: Option<LuaFunction>,
    components_uninstall: Option<LuaFunction>,

    tools_get_buttons: Option<LuaFunction>,

    settings_get_layout: Option<LuaFunction>,
    settings_get_property: Option<LuaFunction>,
    settings_set_property: Option<LuaFunction>
}

impl GameIntegration {
    /// Try to load game integration from provided lua engine and integration
    /// value.
    pub fn load(
        lua: Lua,
        integration: &LuaValue
    ) -> Result<Self, LuaError> {
        let integration = if let Some(table) = integration.as_table() {
            table.clone()
        }

        else if let Some(func) = integration.as_function() {
            let env = lua.create_table_with_capacity(0, 2)?;

            env.raw_set("version", crate::VERSION)?;

            if let Some(platform) = Platform::current() {
                env.raw_set("platform", platform.to_string())?;
            }

            func.call::<LuaTable>(env)?
        }

        else {
            return Err(LuaError::external("invalid game integration format"));
        };

        let game = integration.get::<LuaTable>("game")?;
        let components = integration.get::<LuaTable>("components").ok();
        let tools = integration.get::<LuaTable>("tools").ok();
        let settings = integration.get::<LuaTable>("settings").ok();

        Ok(Self {
            lua,

            game_get_editions: game.get("get_editions").ok(),

            game_get_launch_info: game.get("get_launch_info")
                .context("game.get_launch_info API function must be specified")?,

            game_get_actions_pipeline: game.get("get_actions_pipeline")
                .context("game.get_actions_pipeline API function must be specified")?,

            components_get_layout: components.as_ref()
                .map(|components| components.get("get_layout"))
                .transpose()
                .context("components.get_layout API function must be specified")?,

            components_get_enabled: components.as_ref()
                .map(|components| components.get("get_enabled"))
                .transpose()
                .context("components.get_enabled API function must be specified")?,

            components_set_enabled: components.as_ref()
                .map(|components| components.get("set_enabled"))
                .transpose()
                .context("components.set_enabled API function must be specified")?,

            components_install: components.as_ref()
                .map(|components| {
                    // Both can be accepted for now (word "download" is
                    // reserved), but only "install" is the correct one.
                    components.get::<Option<LuaFunction>>("install")
                        .or_else(|_| components.get::<Option<LuaFunction>>("download"))
                })
                .transpose()?
                .flatten(),

            components_uninstall: components.as_ref()
                .map(|components| {
                    // Both can be accepted for now (word "delete" is reserved),
                    // but only "uninstall" is the correct one.
                    components.get::<Option<LuaFunction>>("uninstall")
                        .or_else(|_| components.get::<Option<LuaFunction>>("delete"))
                })
                .transpose()?
                .flatten(),

            tools_get_buttons: tools.as_ref()
                .map(|tools| tools.get("get_buttons"))
                .transpose()
                .context("tools.get_buttons API function must be specified")?,

            settings_get_layout: settings.as_ref()
                .map(|settings| settings.get("get_layout"))
                .transpose()
                .context("settings.get_layout API function must be specified")?,

            settings_get_property: settings.as_ref()
                .map(|settings| settings.get("get_property"))
                .transpose()
                .context("settings.get_property API function must be specified")?,

            settings_set_property: settings.as_ref()
                .map(|settings| settings.get("set_property"))
                .transpose()
                .context("settings.set_property API function must be specified")?
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
        variant: impl AsRef<GameVariant>
    ) -> Result<Option<GameLaunchInfo>, LuaError> {
        let variant = variant.as_ref()
            .to_lua(&self.lua)?;

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
        variant: impl AsRef<GameVariant>
    ) -> Result<Option<ActionsPipeline>, LuaError> {
        let variant = variant.as_ref()
            .to_lua(&self.lua)?;

        self.game_get_actions_pipeline.call::<Option<LuaTable>>(variant)
            .and_then(|pipeline| {
                pipeline.map(|pipeline| {
                    ActionsPipeline::from_lua(self.lua.clone(), &pipeline)
                }).transpose()
            })
    }

    /// Get game components layout.
    ///
    /// Return `Ok(None)` if components are not specified.
    pub fn get_components_layout(
        &self,
        variant: impl AsRef<GameVariant>
    ) -> Result<Option<Box<[GameComponentsGroup]>>, LuaError> {
        let Some(get_layout) = &self.components_get_layout else {
            return Ok(None);
        };

        let variant = variant.as_ref()
            .to_lua(&self.lua)?;

        get_layout.call::<Vec<LuaTable>>(variant)
            .and_then(|groups| {
                groups.iter()
                    .map(GameComponentsGroup::from_lua)
                    .collect::<Result<Box<[_]>, LuaError>>()
            })
            .map(Some)
    }

    /// Check if given component is enabled.
    ///
    /// Return `Ok(None)` if the function is not specified.
    pub fn get_component_enabled(
        &self,
        variant: impl AsRef<GameVariant>,
        component: impl AsRef<str>
    ) -> Result<Option<bool>, LuaError> {
        let Some(get_enabled) = &self.components_get_enabled else {
            return Ok(None);
        };

        get_enabled.call::<bool>((
            variant.as_ref().to_lua(&self.lua)?,
            component.as_ref()
        )).map(Some)
    }

    /// Enable or disable given component name.
    pub fn set_component_enabled(
        &self,
        variant: impl AsRef<GameVariant>,
        component: impl AsRef<str>,
        enabled: bool
    ) -> Result<(), LuaError> {
        let Some(set_enabled) = &self.components_set_enabled else {
            return Ok(());
        };

        set_enabled.call::<()>((
            variant.as_ref().to_lua(&self.lua)?,
            component.as_ref(),
            enabled
        ))?;

        Ok(())
    }

    /// Install a game component.
    pub fn install_component(
        &self,
        variant: impl AsRef<GameVariant>,
        component: impl AsRef<str>,
        progress: impl Fn(ProgressReport) + Send + 'static
    ) -> Result<(), LuaError> {
        let Some(install_component) = &self.components_install else {
            return Ok(());
        };

        let progress = self.lua.create_function(move |_, report: LuaTable| {
            progress(ProgressReport::from_lua(&report)?);

            Ok(())
        })?;

        install_component.call::<()>((
            variant.as_ref().to_lua(&self.lua)?,
            component.as_ref(),
            progress
        ))?;

        Ok(())
    }

    /// Uninstall a game component.
    pub fn uninstall_component(
        &self,
        variant: impl AsRef<GameVariant>,
        component: impl AsRef<str>,
        progress: impl Fn(ProgressReport) + Send + 'static
    ) -> Result<(), LuaError> {
        let Some(uninstall_component) = &self.components_uninstall else {
            return Ok(());
        };

        let progress = self.lua.create_function(move |_, report: LuaTable| {
            progress(ProgressReport::from_lua(&report)?);

            Ok(())
        })?;

        uninstall_component.call::<()>((
            variant.as_ref().to_lua(&self.lua)?,
            component.as_ref(),
            progress
        ))?;

        Ok(())
    }

    /// Get list of tool buttons from the game integration module.
    ///
    /// Return `Ok(None)` if tools are not specified. Otherwise return list of
    /// information about tool buttons.
    pub fn get_tools_buttons(
        &self,
        variant: impl AsRef<GameVariant>
    ) -> Result<Option<Box<[ToolButton]>>, LuaError> {
        let Some(get_buttons) = &self.tools_get_buttons else {
            return Ok(None);
        };

        let variant = variant.as_ref()
            .to_lua(&self.lua)?;

        let buttons = get_buttons.call::<Vec<LuaTable>>(variant)?
            .iter()
            .map(ToolButton::from_lua)
            .collect::<Result<Box<[ToolButton]>, LuaError>>()?;

        Ok(Some(buttons))
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

        let variant = variant.as_ref()
            .to_lua(&self.lua)?;

        get_layout.call::<Vec<LuaTable>>(variant)
            .and_then(|groups| {
                groups.iter()
                    .map(GameSettingsGroup::from_lua)
                    .collect::<Result<Box<[_]>, LuaError>>()
            })
            .map(Some)
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
}
