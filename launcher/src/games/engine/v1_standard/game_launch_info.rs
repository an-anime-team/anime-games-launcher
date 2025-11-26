use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use mlua::prelude::*;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameLaunchInfo {
    /// Optional status of the game launch button.
    pub status: GameLaunchStatus,

    /// Optional hint displayed nearby the launch button.
    pub hint: Option<LocalizableString>,

    /// Path to the binary.
    pub binary: PathBuf,

    /// Arguments for the binary.
    pub args: Option<Vec<String>>,

    /// Environment variables applied for the binary.
    pub env: Option<HashMap<String, String>>
}

impl Default for GameLaunchInfo {
    fn default() -> Self {
        Self {
            status: GameLaunchStatus::Disabled,
            hint: None,
            binary: PathBuf::new(),
            args: None,
            env: None
        }
    }
}

impl AsLua for GameLaunchInfo {
    fn to_lua(&self, lua: &Lua) -> Result<LuaValue, AsLuaError> {
        let table = lua.create_table_with_capacity(0, 5)?;

        table.set("status", lua.create_string(self.status.to_string())?)?;
        table.set("binary", lua.create_string(self.binary.to_string_lossy().to_string())?)?;

        if let Some(hint) = &self.hint {
            table.set("hint", hint.to_lua(lua)?)?;
        }

        if let Some(args) = &self.args {
            let lua_args = lua.create_table_with_capacity(args.len(), 0)?;

            for arg in args {
                lua_args.push(lua.create_string(arg)?)?;
            }

            table.set("args", lua_args)?;
        }

        if let Some(env) = &self.env {
            let lua_env = lua.create_table_with_capacity(0, env.len())?;

            for (k, v) in env {
                lua_env.set(lua.create_string(k)?, lua.create_string(v)?)?;
            }

            table.set("env", lua_env)?;
        }

        Ok(LuaValue::Table(table))
    }

    fn from_lua(value: &LuaValue) -> Result<Self, AsLuaError> where Self: Sized {
        let value = value.as_table()
            .ok_or_else(|| AsLuaError::InvalidFieldValue("<game launch info>"))?;

        Ok(Self {
            status: value.get::<LuaString>("status")
                .map(|status| GameLaunchStatus::from_str(&status.to_string_lossy()))
                .unwrap_or_else(|_| Ok(GameLaunchStatus::default()))?,

            hint: value.get::<LuaValue>("hint")
                .map(|hint| LocalizableString::from_lua(&hint).map(Some))
                .unwrap_or(Ok(None))?,

            binary: value.get::<LuaString>("binary")
                .map(|binary| PathBuf::from(binary.to_string_lossy().to_string()))?,

            args: value.get::<Vec<LuaString>>("args")
                .map(|args| {
                    args.into_iter()
                        .map(|arg| arg.to_string_lossy().to_string())
                        .collect::<Vec<String>>()
                })
                .map(Some)
                .unwrap_or_default(),

            env: value.get::<LuaTable>("env")
                .map(|env| {
                    env.pairs::<LuaString, LuaString>()
                        .map(|pair| {
                            pair.map(|(key, value)| {
                                let key = key.to_string_lossy().to_string();
                                let value = value.to_string_lossy().to_string();

                                (key, value)
                            })
                        })
                        .collect::<Result<HashMap<_, _>, LuaError>>()
                        .map(Some)
                })
                .unwrap_or(Ok(None))?
        })
    }
}
