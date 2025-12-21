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

#[derive(Debug, Clone)]
pub struct ProgressReport {
    /// Current progress.
    current: u64,

    /// Total progress.
    total: u64,

    /// Optional progress formatting function.
    format: Option<LuaFunction>
}

impl ProgressReport {
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaValue, LuaError> {
        let progress = lua.create_table_with_capacity(0, 3)?;

        progress.raw_set("current", self.current)?;
        progress.raw_set("total", self.total)?;

        if let Some(format) = &self.format {
            progress.raw_set("format", format)?;
        }

        Ok(LuaValue::Table(progress))
    }

    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            current: value.get("current")?,
            total: value.get("total")?,
            format: value.get::<LuaFunction>("format").ok()
        })
    }

    #[inline(always)]
    pub const fn current(&self) -> u64 {
        self.current
    }

    #[inline(always)]
    pub const fn total(&self) -> u64 {
        self.total
    }

    /// Return `current / total` fraction with some safety guarantees.
    pub fn fraction(&self) -> f64 {
        if self.current == 0 {
            return 0.0;
        }

        if self.total == 0 {
            return 1.0;
        }

        self.current as f64 / self.total as f64
    }

    /// Return formatted progress string by using provided formatting callback.
    /// If callback is not provided, then `Ok(None)` is returned.
    pub fn format(&self) -> Result<Option<LocalizableString>, LuaError> {
        let Some(format) = &self.format else {
            return Ok(None);
        };

        let str = format.call::<LuaValue>(())?;

        Ok(Some(LocalizableString::from_lua(&str)?))
    }
}
