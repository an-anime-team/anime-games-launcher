use std::str::FromStr;

use mlua::prelude::*;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameVariant {
    pub platform: TargetPlatform,
    pub edition: String
}

impl GameVariant {
    /// Create game variant struct using current system's platform
    /// and provided game edition.
    pub fn from_edition(edition: impl ToString) -> Self {
        Self {
            platform: *CURRENT_PLATFORM,
            edition: edition.to_string()
        }
    }
}

impl<'lua> AsLua<'lua> for GameVariant {
    fn to_lua(&self, lua: &'lua Lua) -> Result<LuaValue<'lua>, AsLuaError> {
        let table = lua.create_table_with_capacity(0, 2)?;

        table.set("platform", self.platform.to_string())?;
        table.set("edition", self.edition.as_str())?;

        Ok(LuaValue::Table(table))
    }

    fn from_lua(value: &'lua LuaValue<'lua>) -> Result<Self, AsLuaError> where Self: Sized {
        let value = value.as_table()
            .ok_or_else(|| AsLuaError::InvalidFieldValue("<game variant>"))?;

        Ok(Self {
            platform: value.get::<_, LuaString>("platform")
                .map(|platform| TargetPlatform::from_str(&platform.to_string_lossy()))?
                .map_err(|_| AsLuaError::InvalidFieldValue("platform"))?,

            edition: value.get::<_, LuaString>("edition")
                .map(|edition| edition.to_string_lossy().to_string())
                .map_err(|_| AsLuaError::InvalidFieldValue("edition"))?
        })
    }
}
