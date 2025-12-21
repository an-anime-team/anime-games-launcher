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

use agl_locale::LocalizableString;

use super::PipelineAction;

#[derive(Debug, Clone)]
pub struct ActionsPipeline {
    title: LocalizableString,
    description: Option<LocalizableString>,
    pipeline: Box<[PipelineAction]>
}

impl ActionsPipeline {
    pub fn from_lua(lua: Lua, table: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
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

            pipeline: table.get::<Vec<LuaTable>>("pipeline")
                .and_then(|pipeline| {
                    pipeline.iter()
                        .map(|action| PipelineAction::from_lua(lua.clone(), action))
                        .collect::<Result<Box<[_]>, _>>()
                })?
        })
    }

    /// Title of the actions pipeline.
    #[inline(always)]
    pub const fn title(&self) -> &LocalizableString {
        &self.title
    }

    /// Optional description of the actions pipeline.
    #[inline(always)]
    pub const fn description(&self) -> Option<&LocalizableString> {
        self.description.as_ref()
    }

    /// List of actions which will be executed to apply the pipeline.
    #[inline(always)]
    pub const fn actions(&self) -> &[PipelineAction] {
        &self.pipeline
    }
}
