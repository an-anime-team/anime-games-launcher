// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
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

#[derive(Debug, Clone, PartialEq)]
pub struct GameSettingsGroup {
    title: Option<LocalizableString>,
    description: Option<LocalizableString>,
    entries: Box<[GameSettingsEntry]>
}

impl GameSettingsGroup {
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
                .map(GameSettingsEntry::from_lua)
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
    pub const fn entries(&self) -> &[GameSettingsEntry] {
        &self.entries
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameSettingsEntry {
    name: Option<String>,
    title: LocalizableString,
    description: Option<LocalizableString>,
    reactivity: Option<GameSettingsEntryReactivity>,
    entry: GameSettingsEntryFormat
}

impl GameSettingsEntry {
    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            name: value.get::<LuaValue>("name")
                .map(|name| -> Result<Option<String>, LuaError> {
                    if name.is_nil() || name.is_null() {
                        Ok(None)
                    } else {
                        Ok(name.as_string().map(|name| name.to_string_lossy()))
                    }
                })
                .unwrap_or(Ok(None))?,

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

            reactivity: value.get::<String>("reactivity")
                .map(|reactivity| {
                    GameSettingsEntryReactivity::from_str(&reactivity)
                        .map_err(|_| LuaError::external("invalid settings entry reactivity value"))
                        .map(Some)
                })
                .unwrap_or(Ok(None))?,

            entry: value.get::<LuaTable>("entry")
                .and_then(|entry| GameSettingsEntryFormat::from_lua(&entry))?
        })
    }

    #[inline(always)]
    pub const fn name(&self) -> Option<&String> {
        self.name.as_ref()
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
    pub const fn reactivity(&self) -> Option<&GameSettingsEntryReactivity> {
        self.reactivity.as_ref()
    }

    #[inline(always)]
    pub const fn entry(&self) -> &GameSettingsEntryFormat {
        &self.entry
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSettingsEntryReactivity {
    /// Do not refresh game status after changing this entry.
    None,

    /// Refresh game status after closing the settings window.
    #[default]
    Relaxed,

    /// Reload whole settings window immediately after changing this entry
    /// and refresh game status after closing it.
    Release
}

impl std::fmt::Display for GameSettingsEntryReactivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None    => f.write_str("none"),
            Self::Relaxed => f.write_str("relaxed"),
            Self::Release => f.write_str("release")
        }
    }
}

impl FromStr for GameSettingsEntryReactivity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none"    => Ok(Self::None),
            "relaxed" => Ok(Self::Relaxed),
            "release" => Ok(Self::Release),

            _ => Err(())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameSettingsEntryFormat {
    Switch {
        value: bool
    },

    Text {
        value: String
    },

    SecretText {
        value: String
    },

    Number {
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
        value: f64
    },

    Enum {
        /// Vector instead of a HashMap to preserve original order.
        values: Box<[(String, LocalizableString)]>,

        /// Name of selected value.
        selected: String
    },

    Selector {
        /// Vector instead of a HashMap to preserve original order.
        values: Box<[(String, LocalizableString)]>,

        /// Name of selected value.
        selected: String
    },

    Expandable {
        entries: Box<[GameSettingsEntry]>
    }
}

impl GameSettingsEntryFormat {
    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        let format = value.get::<String>("format")?;

        match format.as_str() {
            "switch" => Ok(Self::Switch {
                value: value.get("value")?
            }),

            "text" => Ok(Self::Text {
                value: value.get("value")?
            }),

            // All three values are supported although only "secret_text" is
            // considered standard.
            "secret_text" | "secret" | "password" => Ok(Self::SecretText {
                value: value.get("value")?
            }),

            "number" => Ok(Self::Number {
                min: value.get("min")?,
                max: value.get("max")?,
                step: value.get("step")?,
                value: value.get("value")?
            }),

            "enum" => Ok(Self::Enum {
                values: value.get::<LuaTable>("values")
                    .and_then(|values| {
                        let mut table = Vec::with_capacity(values.raw_len());

                        // Old format (name -> title table).
                        for pair in values.pairs::<LuaValue, LuaValue>() {
                            let pair = pair?;

                            // Skip integer fields (sequence values).
                            if let LuaValue::String(name) = pair.0 {
                                table.push((
                                    name.to_string_lossy(),
                                    LocalizableString::from_lua(&pair.1)?
                                ));
                            }
                        }

                        #[cfg(feature = "tracing")]
                        if !table.is_empty() {
                            tracing::warn!("using outdated enum settings entry values syntax");
                        }

                        // Sort old format entries by their title since
                        // hashmap-like tables don't preserve original items
                        // order.
                        table.sort_by(|a, b| {
                            b.1.default_translation()
                                .cmp(a.1.default_translation())
                        });

                        // New format ({ name, title } objects).
                        for value in values.sequence_values::<LuaTable>() {
                            let value = value?;

                            // Support "key" field although only "name" is
                            // correct.
                            let name = value.raw_get::<String>("name")
                                .or_else(|_| value.raw_get::<String>("key"))?;

                            // Support "value" field although only "title" is
                            // correct.
                            let title = value.raw_get::<LuaValue>("title")
                                .or_else(|_| value.raw_get::<LuaValue>("value"))?;

                            table.push((
                                name,
                                LocalizableString::from_lua(&title)?
                            ));
                        }

                        Ok(table.into_boxed_slice())
                    })?,

                // Support "value" field for consistency, although "selected"
                // is the only correct one.
                selected: value.get("selected")
                    .or_else(|_| value.get("value"))?
            }),

            "selector" => Ok(Self::Selector {
                values: value.get::<LuaTable>("values")
                    .and_then(|values| {
                        let mut table = Vec::with_capacity(values.raw_len());

                        // Old format (name -> title table).
                        for pair in values.pairs::<LuaValue, LuaValue>() {
                            let pair = pair?;

                            // Skip integer fields (sequence values).
                            if let LuaValue::String(name) = pair.0 {
                                table.push((
                                    name.to_string_lossy(),
                                    LocalizableString::from_lua(&pair.1)?
                                ));
                            }
                        }

                        #[cfg(feature = "tracing")]
                        if !table.is_empty() {
                            tracing::warn!("using outdated selector settings entry values syntax");
                        }

                        // Sort old format entries by their title since
                        // hashmap-like tables don't preserve original items
                        // order.
                        table.sort_by(|a, b| {
                            b.1.default_translation()
                                .cmp(a.1.default_translation())
                        });

                        // New format ({ name, title } objects).
                        for value in values.sequence_values::<LuaTable>() {
                            let value = value?;

                            // Support "key" field although only "name" is
                            // correct.
                            let name = value.raw_get::<String>("name")
                                .or_else(|_| value.raw_get::<String>("key"))?;

                            // Support "value" field although only "title" is
                            // correct.
                            let title = value.raw_get::<LuaValue>("title")
                                .or_else(|_| value.raw_get::<LuaValue>("value"))?;

                            table.push((
                                name,
                                LocalizableString::from_lua(&title)?
                            ));
                        }

                        Ok(table.into_boxed_slice())
                    })?,

                // Support "value" field for consistency, although "selected"
                // is the only correct one.
                selected: value.get("selected")
                    .or_else(|_| value.get("value"))?
            }),

            "expandable" => Ok(Self::Expandable {
                entries: value.get::<Vec<LuaTable>>("entries")
                    .and_then(|entries| {
                        entries.iter()
                            .map(GameSettingsEntry::from_lua)
                            .collect::<Result<Box<[_]>, LuaError>>()
                    })?
            }),

            _ => Err(LuaError::external("unsupported settings entry format"))
        }
    }
}
