use mlua::prelude::*;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameEdition {
    /// Unique name of the edition.
    pub name: String,

    /// Title used in UI.
    pub title: LocalizableString
}

impl TryFrom<&LuaTable> for GameEdition {
    type Error = LuaError;

    fn try_from(value: &LuaTable) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.get::<LuaString>("name")?
                .to_string_lossy()
                .to_string(),

            title: value.get::<LuaValue>("title")
                .map_err(AsLuaError::LuaError)
                .and_then(|title| LocalizableString::from_lua(&title))?
        })
    }
}
