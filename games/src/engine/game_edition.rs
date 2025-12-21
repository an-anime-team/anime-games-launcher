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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEdition {
    /// Unique name of the edition.
    pub name: String,

    /// Title used in UI.
    pub title: LocalizableString
}

impl GameEdition {
    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            name: value.get::<String>("name")?,

            title: value.get::<LuaValue>("title")
                .and_then(|title| LocalizableString::from_lua(&title))?
        })
    }
}
