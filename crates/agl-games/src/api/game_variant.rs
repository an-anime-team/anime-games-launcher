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

use crate::platform::Platform;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameVariant {
    pub platform: Platform,
    pub edition: Option<String>
}

impl GameVariant {
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaTable, LuaError> {
        let table = lua.create_table_with_capacity(0, 2)?;

        table.raw_set("platform", self.platform.to_string())?;

        if let Some(edition) = self.edition.as_deref() {
            table.raw_set("edition", edition)?;
        }

        Ok(table)
    }

    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            platform: value.get::<String>("platform")
                .and_then(|platform| {
                    Platform::from_str(&platform)
                        .map_err(|_| LuaError::external("invalid platform format"))
                })?,

            edition: value.get::<Option<String>>("edition").ok().flatten()
        })
    }
}

impl AsRef<GameVariant> for GameVariant {
    #[inline(always)]
    fn as_ref(&self) -> &GameVariant {
        self
    }
}
