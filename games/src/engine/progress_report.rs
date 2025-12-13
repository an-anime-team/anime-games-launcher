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

#[derive(Debug, Clone)]
pub struct ProgressReport {
    /// Optional title of the current action.
    pub title: Option<LocalizableString>,

    /// Optional description of the current action.
    pub description: Option<LocalizableString>,

    /// Current progress.
    pub progress_current: u64,

    /// Total progress.
    pub progress_total: u64,

    /// Optional progress formatting function.
    progress_format: Option<LuaFunction>
}

impl ProgressReport {
    /// Return `current / total` fraction with some safety guarantees.
    pub fn fraction(&self) -> f64 {
        if self.progress_current == 0 {
            return 0.0;
        }

        if self.progress_total == 0 {
            return 1.0;
        }

        self.progress_current as f64 / self.progress_total as f64
    }

    /// Return formatted progress string by using provided formatting callback.
    /// If callback is not provided, then `Ok(None)` is returned.
    pub fn format(&self) -> Result<Option<LocalizableString>, LuaError> {
        let Some(format) = &self.progress_format else {
            return Ok(None);
        };

        let str = format.call::<LuaValue>(())?;

        Ok(Some(LocalizableString::from_lua(&str)?))
    }

    pub fn to_lua(&self, lua: &Lua) -> Result<LuaValue, LuaError> {
        let progress = lua.create_table_with_capacity(0, 3)?;

        if let Some(title) = &self.title {
            progress.raw_set("title", title.to_lua(lua)?)?;
        }

        if let Some(description) = &self.description {
            progress.raw_set("description", description.to_lua(lua)?)?;
        }

        let progress_details = lua.create_table_with_capacity(0, 3)?;

        progress_details.raw_set("current", self.progress_current)?;
        progress_details.raw_set("total", self.progress_total)?;

        if let Some(format) = &self.progress_format {
            progress_details.raw_set("format", format)?;
        }

        progress.raw_set("progress", progress_details)?;

        Ok(LuaValue::Table(progress))
    }

    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        let progress = value.get::<LuaTable>("progress")?;

        Ok(Self {
            title: value.get::<LuaValue>("title")
                .map(|title| -> Result<Option<LocalizableString>, LuaError> {
                    if title.is_nil() || title.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&title)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            description: value.get::<LuaValue>("description")
                .map(|title| -> Result<Option<LocalizableString>, LuaError> {
                    if title.is_nil() || title.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&title)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            progress_current: progress.get("current")?,
            progress_total: progress.get("total")?,

            progress_format: progress.get::<LuaFunction>("format").ok()
        })
    }
}
