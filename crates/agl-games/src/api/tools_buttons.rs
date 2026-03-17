// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
// Copyright (C) 2026  Nikita Podvirnyi <krypt0nn@vk.com>
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

use agl_locale::string::LocalizableString;

#[derive(Debug, Clone)]
pub struct ToolButton {
    title: LocalizableString,
    description: Option<LocalizableString>,
    callback: LuaFunction
}

impl ToolButton {
    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            title: value.get::<LuaValue>("title")
                .and_then(|title| LocalizableString::from_lua(&title))?,

            description: value.get::<LuaValue>("description")
                .map(|desc| -> Result<Option<LocalizableString>, LuaError> {
                    if desc.is_nil() || desc.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&desc)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            callback: value.get("callback")?
        })
    }

    #[inline(always)]
    pub const fn title(&self) -> &LocalizableString {
        &self.title
    }

    #[inline(always)]
    pub const fn description(&self) -> Option<&LocalizableString> {
        self.description.as_ref()
    }

    pub fn call(&self) -> Result<(), LuaError> {
        self.callback.call::<()>(())?;

        Ok(())
    }
}
