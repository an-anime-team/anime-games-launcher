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

#[derive(Debug, Clone, PartialEq)]
pub struct GameComponentsGroup {
    title: Option<LocalizableString>,
    description: Option<LocalizableString>,
    entries: Box<[GameComponentsEntry]>
}

impl GameComponentsGroup {
    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            title: value.raw_get::<LuaValue>("title")
                .map(|title| -> Result<Option<LocalizableString>, LuaError> {
                    if title.is_nil() || title.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&title)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            description: value.raw_get::<LuaValue>("description")
                .map(|desc| -> Result<Option<LocalizableString>, LuaError> {
                    if desc.is_nil() || desc.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&desc)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            entries: value.raw_get::<Vec<LuaTable>>("entries")?
                .iter()
                .map(GameComponentsEntry::from_lua)
                .collect::<Result<Box<[_]>, LuaError>>()?
        })
    }

    #[inline(always)]
    pub const fn title(&self) -> Option<&LocalizableString> {
        self.title.as_ref()
    }

    #[inline(always)]
    pub const fn description(&self) -> Option<&LocalizableString> {
        self.description.as_ref()
    }

    #[inline(always)]
    pub const fn entries(&self) -> &[GameComponentsEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameComponentsEntry {
    name: String,
    title: LocalizableString,
    description: Option<LocalizableString>,
    locked: bool
}

impl GameComponentsEntry {
    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            name: value.get::<String>("name")?,

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

            locked: value.get::<Option<bool>>("locked").ok()
                .flatten()
                .unwrap_or(false)
        })
    }

    #[inline(always)]
    pub const fn name(&self) -> &str {
        self.name.as_str()
    }

    #[inline(always)]
    pub const fn title(&self) -> &LocalizableString {
        &self.title
    }

    #[inline(always)]
    pub const fn description(&self) -> Option<&LocalizableString> {
        self.description.as_ref()
    }

    #[inline(always)]
    pub const fn is_locked(&self) -> bool {
        self.locked
    }
}
