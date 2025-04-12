use std::collections::HashMap;

use mlua::prelude::*;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSettingsGroup {
    pub title: Option<LocalizableString>,
    pub description: Option<LocalizableString>,
    pub entries: Vec<GameSettingsEntry>
}

impl<'lua> AsLua<'lua> for GameSettingsGroup {
    fn to_lua(&self, lua: &'lua Lua) -> Result<LuaValue<'lua>, AsLuaError> {
        let table = lua.create_table_with_capacity(0, 3)?;

        if let Some(title) = &self.title {
            table.set("title", title.to_lua(lua)?)?;
        }

        if let Some(desc) = &self.description {
            table.set("description", desc.to_lua(lua)?)?;
        }

        let entries = lua.create_table_with_capacity(self.entries.len(), 0)?;

        for entry in &self.entries {
            entries.push(entry.to_lua(lua)?)?;
        }

        table.set("entries", entries)?;

        Ok(LuaValue::Table(table))
    }

    fn from_lua(value: &'lua LuaValue<'lua>) -> Result<Self, AsLuaError> where Self: Sized {
        let value = value.as_table()
            .ok_or_else(|| AsLuaError::InvalidFieldValue("settings[]"))?;

        Ok(Self {
            title: value.get::<_, LuaValue>("title")
                .map(|title| -> Result<Option<LocalizableString>, AsLuaError> {
                    if title.is_nil() || title.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&title)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            description: value.get::<_, LuaValue>("description")
                .map(|desc| -> Result<Option<LocalizableString>, AsLuaError> {
                    if desc.is_nil() || desc.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&desc)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            entries: value.get::<_, Vec<LuaValue>>("entries")
                .map_err(|_| AsLuaError::InvalidFieldValue("settings[].entries"))?
                .iter()
                .map(GameSettingsEntry::from_lua)
                .collect::<Result<Vec<_>, _>>()?
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSettingsEntry {
    pub name: Option<String>,
    pub title: LocalizableString,
    pub description: Option<LocalizableString>,
    pub entry: GameSettingsEntryFormat
}

impl<'lua> AsLua<'lua> for GameSettingsEntry {
    fn to_lua(&self, lua: &'lua Lua) -> Result<LuaValue<'lua>, AsLuaError> {
        let table = lua.create_table_with_capacity(0, 4)?;

        table.set("title", self.title.to_lua(lua)?)?;
        table.set("entry", self.entry.to_lua(lua)?)?;

        if let Some(name) = &self.name {
            table.set("name", lua.create_string(name)?)?;
        }

        if let Some(desc) = &self.description {
            table.set("description", desc.to_lua(lua)?)?;
        }

        Ok(LuaValue::Table(table))
    }

    fn from_lua(value: &'lua LuaValue<'lua>) -> Result<Self, AsLuaError> where Self: Sized {
        let value = value.as_table()
            .ok_or_else(|| AsLuaError::InvalidFieldValue("settings.entries[]"))?;

        Ok(Self {
            name: value.get::<_, LuaValue>("name")
                .map(|name| -> Result<Option<String>, AsLuaError> {
                    if name.is_nil() || name.is_null() {
                        Ok(None)
                    } else {
                        Ok(name.as_string_lossy().map(String::from))
                    }
                })
                .unwrap_or(Ok(None))?,

            title: value.get::<_, LuaValue>("title")
                .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].title"))
                .and_then(|title| LocalizableString::from_lua(&title))?,

            description: value.get::<_, LuaValue>("description")
                .map(|desc| -> Result<Option<LocalizableString>, AsLuaError> {
                    if desc.is_nil() || desc.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&desc)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            entry: value.get::<_, LuaValue>("entry")
                .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].entry"))
                .and_then(|title| GameSettingsEntryFormat::from_lua(&title))?
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameSettingsEntryFormat {
    Switch {
        value: bool
    },

    Text {
        value: String
    },

    Enum {
        /// Vector instead of a HashMap to preserve original order.
        values: Vec<(String, LocalizableString)>,

        selected: String
    },

    Expandable {
        entries: Vec<GameSettingsEntry>
    }
}

impl<'lua> AsLua<'lua> for GameSettingsEntryFormat {
    fn to_lua(&self, lua: &'lua Lua) -> Result<LuaValue<'lua>, AsLuaError> {
        let table = lua.create_table_with_capacity(0, 3)?;

        match self {
            Self::Switch { value } => {
                table.set("format", "switch")?;
                table.set("value", *value)?;
            }

            Self::Text { value } => {
                table.set("format", "text")?;
                table.set("value", lua.create_string(value)?)?;
            }

            Self::Enum { values, selected } => {
                let enum_values = lua.create_table_with_capacity(0, values.len())?;

                table.set("format", "enum")?;
                table.set("values", enum_values.clone())?;
                table.set("selected", lua.create_string(selected)?)?;

                for (key, value) in values {
                    enum_values.set(key.as_str(), value.to_lua(lua)?)?;
                }
            }

            Self::Expandable { entries } => {
                let row_entries = lua.create_table_with_capacity(entries.len(), 0)?;

                table.set("format", "expandable")?;
                table.set("entries", row_entries.clone())?;

                for entry in entries {
                    row_entries.push(entry.to_lua(lua)?)?;
                }
            }
        }

        Ok(LuaValue::Table(table))
    }

    fn from_lua(value: &'lua LuaValue<'lua>) -> Result<Self, AsLuaError> where Self: Sized {
        let value = value.as_table()
            .ok_or_else(|| AsLuaError::InvalidFieldValue("settings.entries[].entry"))?;

        let format = value.get::<_, LuaString>("format")
            .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].entry.format"))?;

        match format.as_bytes() {
            b"switch" => Ok(Self::Switch {
                value: value.get("value")
                    .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].entry.value"))?
            }),

            b"text" => Ok(Self::Text {
                value: value.get("value")
                    .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].entry.value"))?
            }),

            b"enum" => Ok(Self::Enum {
                values: value.get::<_, LuaTable>("values")
                    .and_then(|values| {
                        let mut table = Vec::with_capacity(values.len()? as usize);

                        for pair in values.pairs::<LuaString, LuaValue>() {
                            let (key, value) = pair?;

                            table.push((
                                key.to_string_lossy().to_string(),
                                LocalizableString::from_lua(&value)?
                            ));
                        }

                        Ok(table)
                    })
                    .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].entry.values"))?,

                selected: value.get::<_, LuaString>("selected")
                    .map(|selected| selected.to_string_lossy().to_string())
                    .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].entry.selected"))?
            }),

            b"expandable" => Ok(Self::Expandable {
                entries: value.get::<_, Vec<LuaValue>>("entries")
                    .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].entry.entries"))
                    .and_then(|entries| {
                        entries.iter()
                            .map(GameSettingsEntry::from_lua)
                            .collect::<Result<Vec<_>, AsLuaError>>()
                    })?
            }),

            _ => Err(AsLuaError::InvalidFieldValue("settings.entries[].entry.format"))
        }
    }
}
