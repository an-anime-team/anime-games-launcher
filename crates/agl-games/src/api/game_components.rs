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

use std::str::FromStr;

use mlua::prelude::*;

use agl_locale::string::LocalizableString;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameComponentEntryValueStatus {
    #[default]
    Normal,
    Warning,
    Danger,
    Success
}

impl std::fmt::Display for GameComponentEntryValueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal  => f.write_str("normal"),
            Self::Warning => f.write_str("warning"),
            Self::Danger  => f.write_str("danger"),
            Self::Success => f.write_str("success")
        }
    }
}

impl FromStr for GameComponentEntryValueStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal"  | "default"             => Ok(Self::Normal),
            "warning" | "warn"                => Ok(Self::Warning),
            "danger"  | "dangerous" | "error" => Ok(Self::Danger),
            "success"                         => Ok(Self::Success),

            _ => Err(())
        }
    }
}

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
    locked: bool,
    values: Box<[GameComponentsEntryValue]>
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
                .unwrap_or(false),

            values: value.get::<Vec<LuaTable>>("values")
                .map(|values| {
                    values.iter()
                        .map(GameComponentsEntryValue::from_lua)
                        .collect::<Result<Box<[_]>, LuaError>>()
                })
                .unwrap_or(Ok(Box::new([])))?
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

    #[inline(always)]
    pub const fn values(&self) -> &[GameComponentsEntryValue] {
        &self.values
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameComponentsEntryValue {
    title: LocalizableString,
    value: LocalizableString,
    description: Option<LocalizableString>,
    status: GameComponentEntryValueStatus
}

impl GameComponentsEntryValue {
    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            title: value.get::<LuaValue>("title")
                .and_then(|title| LocalizableString::from_lua(&title))?,

            value: value.get::<LuaValue>("value")
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

            status: value.get::<String>("status")
                .map(|status| {
                    GameComponentEntryValueStatus::from_str(&status)
                        .map_err(|_| LuaError::external("invalid game component value status"))
                })
                .unwrap_or_else(|_| Ok(GameComponentEntryValueStatus::default()))?,
        })
    }

    #[inline(always)]
    pub const fn title(&self) -> &LocalizableString {
        &self.title
    }

    #[inline(always)]
    pub const fn value(&self) -> &LocalizableString {
        &self.value
    }

    #[inline(always)]
    pub const fn description(&self) -> Option<&LocalizableString> {
        self.description.as_ref()
    }

    #[inline(always)]
    pub const fn status(&self) -> &GameComponentEntryValueStatus {
        &self.status
    }
}
