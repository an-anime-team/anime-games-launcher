use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use mlua::prelude::*;

use crate::games::prelude::*;

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

impl TryFrom<&LuaTable<'_>> for GameLaunchInfo {
    type Error = LuaError;

    fn try_from(value: &LuaTable<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            status: value.get::<_, LuaString>("status")
                .map(|status| GameLaunchStatus::from_str(&status.to_string_lossy()))
                .unwrap_or_else(|_| Ok(GameLaunchStatus::default()))?,

            hint: value.get::<_, LuaValue>("hint")
                .map(|hint| LocalizableString::try_from(&hint).map(Some))
                .unwrap_or(Ok(None))?,

            binary: value.get::<_, LuaString>("binary")
                .map(|binary| PathBuf::from(binary.to_string_lossy().to_string()))?,

            args: value.get::<_, Vec<LuaString>>("args")
                .map(|args| {
                    args.into_iter()
                        .map(|arg| arg.to_string_lossy().to_string())
                        .collect::<Vec<String>>()
                })
                .map(Some)
                .unwrap_or_default(),

            env: value.get::<_, LuaTable>("env")
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
