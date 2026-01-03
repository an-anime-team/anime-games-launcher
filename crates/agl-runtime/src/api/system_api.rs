// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-runtime
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

use std::path::PathBuf;

use mlua::prelude::*;

pub struct SystemApi {
    lua: Lua,

    system_local_time: LuaFunction,
    system_utc_time: LuaFunction,
    system_env: LuaFunction,
    system_find: LuaFunction
}

impl SystemApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        fn format_time(time: time::OffsetDateTime, format: String) -> Result<String, LuaError> {
            let time = match format.as_str() {
                "iso8601" => time.format(&time::format_description::well_known::Iso8601::DEFAULT),
                "rfc2822" => time.format(&time::format_description::well_known::Rfc2822),
                "rfc3339" => time.format(&time::format_description::well_known::Rfc3339),

                _ => {
                    let format = time::format_description::parse(&format)
                        .map_err(|err| {
                            LuaError::external(format!("failed to parse time formatting: {err}"))
                        })?;

                    time.format(&format)
                }
            };

            let time = time.map_err(|err| {
                LuaError::external(format!("failed to format time: {err}"))
            })?;

            Ok(time)
        }

        Ok(Self {
            system_local_time: lua.create_function(|lua: &Lua, format: Option<String>| {
                let time = time::OffsetDateTime::now_local()
                    .map_err(|err| {
                        LuaError::external(format!("failed to parse local time: {err}"))
                    })?;

                let Some(format) = format else {
                    // Set local time offset as if it was GMT. Required because
                    // `unix_timestamp` method respects timezone and converts it
                    // into UTC.
                    let time = time.replace_offset(time::UtcOffset::UTC)
                        .unix_timestamp();

                    return Ok(LuaValue::Integer(time));
                };

                lua.create_string(format_time(time, format)?)
                    .map(LuaValue::String)
            })?,

            system_utc_time: lua.create_function(|lua: &Lua, format: Option<String>| {
                let time = time::OffsetDateTime::now_utc();

                let Some(format) = format else {
                    return Ok(LuaValue::Integer(time.unix_timestamp()));
                };

                lua.create_string(format_time(time, format)?)
                    .map(LuaValue::String)
            })?,

            system_env: lua.create_function(|lua: &Lua, names: LuaVariadic<String>| {
                if names.is_empty() {
                    let env = lua.create_table_from(std::env::vars())
                        .map(LuaValue::Table)?;

                    Ok(LuaMultiValue::from_vec(vec![env]))
                }

                else {
                    let result = names.into_iter()
                        .map(|name| {
                            std::env::var(&name).ok()
                                .and_then(|value| {
                                    lua.create_string(value).ok()
                                        .map(LuaValue::String)
                                })
                                .unwrap_or(LuaValue::Nil)
                        })
                        .collect();

                    Ok(LuaMultiValue::from_vec(result))
                }
            })?,

            system_find: lua.create_function(|lua: &Lua, names: LuaVariadic<String>| {
                if names.is_empty() {
                    return Ok(LuaMultiValue::new());
                }

                let Ok(paths) = std::env::var("PATH") else {
                    return Err(LuaError::external("failed to read PATH variable"));
                };

                let mut result = vec![LuaValue::Nil; names.len()];

                for path in paths.split(':').map(PathBuf::from) {
                    let mut any = false;

                    for (i, name) in names.iter().enumerate() {
                        if !result[i].is_nil() {
                            continue;
                        }

                        any = true;

                        let path = path.join(name);

                        if path.exists() {
                            let path = lua.create_string(path.as_os_str().as_encoded_bytes())
                                .map(LuaValue::String)?;

                            result[i] = path;
                        }
                    }

                    if !any {
                        break;
                    }
                }

                Ok(LuaMultiValue::from_vec(result))
            })?,

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 4)?;

        env.raw_set("local_time", &self.system_local_time)?;
        env.raw_set("utc_time", &self.system_utc_time)?;
        env.raw_set("env", &self.system_env)?;
        env.raw_set("find", &self.system_find)?;

        Ok(env)
    }
}
