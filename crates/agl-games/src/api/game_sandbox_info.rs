// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
// Copyright (C) 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
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

use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use mlua::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct GameSandboxInfo {
    /// Path to directory where game data should be stored. Files outside of
    /// this directory will not be accessible. Persistent, temporary, and other
    /// runtime directories will be mounted as read-only.
    pub path: PathBuf
}

impl GameSandboxInfo {
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaTable, LuaError> {
        let table = lua.create_table_with_capacity(0, 1)?;

        table.raw_set("path", lua.create_string(
            self.path.as_os_str().as_bytes())?
        )?;

        Ok(table)
    }

    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            path: value.get::<LuaString>("path")
                .map(|path| PathBuf::from(path.to_string_lossy().to_string()))?
        })
    }
}
