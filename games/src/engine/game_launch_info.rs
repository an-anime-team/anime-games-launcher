// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-games
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
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

use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use mlua::prelude::*;

use crate::localizable_string::LocalizableString;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameLaunchStatus {
    #[default]
    Normal,

    Warning,
    Danger,
    Disabled
}

impl std::fmt::Display for GameLaunchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal   => f.write_str("normal"),
            Self::Warning  => f.write_str("warning"),
            Self::Danger   => f.write_str("danger"),
            Self::Disabled => f.write_str("disabled")
        }
    }
}

impl FromStr for GameLaunchStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" | "default"   => Ok(Self::Normal),
            "warning" | "warn"     => Ok(Self::Warning),
            "danger" | "dangerous" => Ok(Self::Danger),
            "disabled" | "disable" => Ok(Self::Disabled),

            _ => Err(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameLaunchInfo {
    /// Status of the game launch button.
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

impl GameLaunchInfo {
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaTable, LuaError> {
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

        Ok(table)
    }

    pub fn from_lua(value: &LuaTable) -> Result<Self, LuaError> {
        Ok(Self {
            status: value.get::<String>("status")
                .map(|status| {
                    GameLaunchStatus::from_str(&status)
                        .map_err(|_| LuaError::external("invalid game launch status"))
                })
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
