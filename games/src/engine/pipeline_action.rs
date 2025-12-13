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

use crate::localizable_string::LocalizableString;

use super::ProgressReport;

#[derive(Debug, Clone)]
pub struct PipelineAction {
    lua: Lua,
    title: LocalizableString,
    description: Option<LocalizableString>,
    before: Option<LuaFunction>,
    perform: LuaFunction
}

impl PipelineAction {
    pub fn from_lua(lua: Lua, table: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            lua,

            title: table.get::<LuaValue>("title")
                .and_then(|title| LocalizableString::from_lua(&title))?,

            description: table.get::<LuaValue>("description")
                .map(|desc| -> Result<Option<LocalizableString>, LuaError> {
                    if desc.is_nil() || desc.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&desc)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            before: table.get::<LuaFunction>("before").ok(),
            perform: table.get("perform")?
        })
    }

    /// Get title of the action.
    #[inline(always)]
    pub const fn title(&self) -> &LocalizableString {
        &self.title
    }

    /// Get optional description of the action.
    #[inline(always)]
    pub const fn description(&self) -> Option<&LocalizableString> {
        self.description.as_ref()
    }

    /// Try to call `before` function if it's specified or return `None`.
    ///
    /// If `Some(true)` is returned, then the action should be started.
    /// Otherwise, if `Some(false)` is returned, then the action should be
    /// skipped.
    pub fn before(
        &self,
        progress: impl Fn(ProgressReport) -> bool + Send + 'static
    ) -> Result<Option<bool>, LuaError> {
        let Some(before) = &self.before else {
            return Ok(None);
        };

        let progress = self.lua.create_function(move |_, report: LuaTable| {
            Ok(progress(ProgressReport::from_lua(&report)?))
        })?;

        before.call::<bool>(progress).map(Some)
    }

    /// Perform the action.
    ///
    /// If `true` is returned, then the next action should be executed.
    /// Otherwise, if `false` is returned, then current action has failed and
    /// pipeline should be stopped.
    pub fn perform(
        &self,
        progress: impl Fn(ProgressReport) + Send + 'static
    ) -> Result<bool, LuaError> {
        let progress = self.lua.create_function(move |_, report: LuaTable| {
            progress(ProgressReport::from_lua(&report)?);

            Ok(())
        })?;

        self.perform.call::<bool>(progress)
    }
}
