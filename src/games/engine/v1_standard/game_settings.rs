use mlua::prelude::*;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSettingsGroup {
    pub title: Option<LocalizableString>,
    pub description: Option<LocalizableString>,
    pub entries: Vec<GameSettingsEntry>
}

impl AsLua for GameSettingsGroup {
    fn to_lua(&self, lua: &Lua) -> Result<LuaValue, AsLuaError> {
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

    fn from_lua(value: &LuaValue) -> Result<Self, AsLuaError> where Self: Sized {
        let value = value.as_table()
            .ok_or_else(|| AsLuaError::InvalidFieldValue("settings[]"))?;

        Ok(Self {
            title: value.get::<LuaValue>("title")
                .map(|title| -> Result<Option<LocalizableString>, AsLuaError> {
                    if title.is_nil() || title.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&title)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            description: value.get::<LuaValue>("description")
                .map(|desc| -> Result<Option<LocalizableString>, AsLuaError> {
                    if desc.is_nil() || desc.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&desc)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            entries: value.get::<Vec<LuaValue>>("entries")
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
    pub reactivity: Option<GameSettingsEntryReactivity>,
    pub entry: GameSettingsEntryFormat
}

impl AsLua for GameSettingsEntry {
    fn to_lua(&self, lua: &Lua) -> Result<LuaValue, AsLuaError> {
        let table = lua.create_table_with_capacity(0, 5)?;

        table.set("title", self.title.to_lua(lua)?)?;
        table.set("entry", self.entry.to_lua(lua)?)?;

        if let Some(name) = self.name.as_ref() {
            table.set("name", lua.create_string(name)?)?;
        }

        if let Some(description) = self.description.as_ref() {
            table.set("description", description.to_lua(lua)?)?;
        }

        if let Some(reactivity) = self.reactivity.as_ref() {
            table.set("reactivity", reactivity.to_lua(lua)?)?;
        }

        Ok(LuaValue::Table(table))
    }

    fn from_lua(value: &LuaValue) -> Result<Self, AsLuaError> where Self: Sized {
        let value = value.as_table()
            .ok_or_else(|| AsLuaError::InvalidFieldValue("settings.entries[]"))?;

        Ok(Self {
            name: value.get::<LuaValue>("name")
                .map(|name| -> Result<Option<String>, AsLuaError> {
                    if name.is_nil() || name.is_null() {
                        Ok(None)
                    } else {
                        Ok(name.as_string_lossy())
                    }
                })
                .unwrap_or(Ok(None))?,

            title: value.get::<LuaValue>("title")
                .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].title"))
                .and_then(|title| LocalizableString::from_lua(&title))?,

            description: value.get::<LuaValue>("description")
                .map(|desc| -> Result<Option<LocalizableString>, AsLuaError> {
                    if desc.is_nil() || desc.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(LocalizableString::from_lua(&desc)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            reactivity: value.get::<LuaValue>("reactivity")
                .map(|reactivity| -> Result<Option<GameSettingsEntryReactivity>, AsLuaError> {
                    if reactivity.is_nil() || reactivity.is_null() {
                        Ok(None)
                    } else {
                        Ok(Some(GameSettingsEntryReactivity::from_lua(&reactivity)?))
                    }
                })
                .unwrap_or(Ok(None))?,

            entry: value.get::<LuaValue>("entry")
                .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].entry"))
                .and_then(|title| GameSettingsEntryFormat::from_lua(&title))?
        })
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

impl AsLua for GameSettingsEntryReactivity {
    fn to_lua(&self, lua: &Lua) -> Result<LuaValue, AsLuaError> {
        let value = match self {
            Self::None    => "none",
            Self::Relaxed => "relaxed",
            Self::Release => "release"
        };

        Ok(LuaValue::String(lua.create_string(value)?))
    }

    fn from_lua(value: &LuaValue) -> Result<Self, AsLuaError> where Self: Sized {
        match value.to_string()?.as_str() {
            "none"    => Ok(Self::None),
            "relaxed" => Ok(Self::Relaxed),
            "release" => Ok(Self::Release),

            _ => Err(AsLuaError::InvalidFieldValue("<entry reactivity>"))
        }
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

impl AsLua for GameSettingsEntryFormat {
    fn to_lua(&self, lua: &Lua) -> Result<LuaValue, AsLuaError> {
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

    fn from_lua(value: &LuaValue) -> Result<Self, AsLuaError> where Self: Sized {
        let value = value.as_table()
            .ok_or_else(|| AsLuaError::InvalidFieldValue("settings.entries[].entry"))?;

        let format = value.get::<LuaString>("format")
            .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].entry.format"))?;

        match format.as_bytes().as_ref() {
            b"switch" => Ok(Self::Switch {
                value: value.get("value")
                    .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].entry.value"))?
            }),

            b"text" => Ok(Self::Text {
                value: value.get("value")
                    .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].entry.value"))?
            }),

            b"enum" => Ok(Self::Enum {
                values: value.get::<LuaTable>("values")
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

                selected: value.get::<LuaString>("selected")
                    .map(|selected| selected.to_string_lossy().to_string())
                    .map_err(|_| AsLuaError::InvalidFieldValue("settings.entries[].entry.selected"))?
            }),

            b"expandable" => Ok(Self::Expandable {
                entries: value.get::<Vec<LuaValue>>("entries")
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
