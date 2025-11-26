use std::str::FromStr;

use mlua::prelude::*;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameVariant {
    pub platform: TargetPlatform,
    pub edition: Option<String>
}

impl Default for GameVariant {
    fn default() -> Self {
        Self {
            platform: *CURRENT_PLATFORM,
            edition: None
        }
    }
}

impl GameVariant {
    /// Create game variant struct using current system's platform
    /// and provided game edition.
    pub fn from_edition(edition: impl ToString) -> Self {
        Self {
            platform: *CURRENT_PLATFORM,
            edition: Some(edition.to_string())
        }
    }
}

impl AsLua for GameVariant {
    fn to_lua(&self, lua: &Lua) -> Result<LuaValue, AsLuaError> {
        let table = lua.create_table_with_capacity(0, 2)?;

        table.raw_set("platform", self.platform.to_string())?;

        if let Some(edition) = self.edition.as_deref() {
            table.raw_set("edition", edition)?;
        }

        Ok(LuaValue::Table(table))
    }

    fn from_lua(value: &LuaValue) -> Result<Self, AsLuaError> where Self: Sized {
        let value = value.as_table()
            .ok_or_else(|| AsLuaError::InvalidFieldValue("<game variant>"))?;

        Ok(Self {
            platform: value.get::<LuaString>("platform")
                .map(|platform| TargetPlatform::from_str(&platform.to_string_lossy()))?
                .map_err(|_| AsLuaError::InvalidFieldValue("platform"))?,

            edition: value.get::<Option<LuaString>>("edition").ok()
                .and_then(|edition| {
                    edition.map(|edition| {
                        edition.to_string_lossy().to_string()
                    })
                })
        })
    }
}

impl AsRef<GameVariant> for GameVariant {
    #[inline(always)]
    fn as_ref(&self) -> &GameVariant {
        self
    }
}
