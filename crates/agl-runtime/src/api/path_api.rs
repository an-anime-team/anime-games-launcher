// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-runtime
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

use mlua::prelude::*;

use super::*;

fn normalize_path_parts(parts: &[impl AsRef<str>]) -> Option<Vec<String>> {
    let mut normal_parts = Vec::with_capacity(parts.len());

    let mut i = 0;
    let n = parts.len();

    while i < n {
        let part = parts[i].as_ref();

        if part == "." {
            i += 1;
        }

        else if part == ".." {
            normal_parts.pop()?;

            i += 1;
        }

        else {
            if !["", "/", "\\"].contains(&part) {
                normal_parts.push(part.to_string());
            }

            i += 1;
        }
    }

    if normal_parts.is_empty() {
        None
    } else {
        Some(normal_parts)
    }
}

fn split_path(path: impl AsRef<str>) -> Option<Vec<String>> {
    let path = path.as_ref()
        .replace('\\', "/");

    let raw_parts = path
        .split('/')
        .collect::<Vec<_>>();

    normalize_path_parts(&raw_parts)
}

pub struct PathApi {
    lua: Lua,

    path_temp_dir: LuaFunctionBuilder,
    path_module_dir: LuaFunctionBuilder,
    path_persist_dir: LuaFunctionBuilder,
    path_normalize: LuaFunction,
    path_join: LuaFunction,
    path_parts: LuaFunction,
    path_parent: LuaFunction,
    path_file_name: LuaFunction,
    path_exists: LuaFunctionBuilder,
    path_permissions: LuaFunctionBuilder
}

impl PathApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        Ok(Self {
            path_temp_dir: Box::new(|lua: &Lua, context: &Context| {
                let path = context.temp_folder.clone();

                lua.create_function(move |_, ()| Ok(path.clone()))
            }),

            path_module_dir: Box::new(|lua: &Lua, context: &Context| {
                let path = context.module_folder.clone();

                lua.create_function(move |_, ()| Ok(path.clone()))
            }),

            path_persist_dir: Box::new(|lua: &Lua, context: &Context| {
                let path = context.persistent_folder.clone();

                lua.create_function(move |_, key: LuaString| {
                    fn normalize_key(key: LuaString) -> String {
                        let hash = seahash::hash(&key.as_bytes());

                        let key = key.to_string_lossy()
                            .chars()
                            .map(|char| {
                                if char.is_ascii_alphanumeric() {
                                    char
                                } else {
                                    '_'
                                }
                            })
                            .collect::<String>();

                        let key = key.trim_matches('_')
                            .replace("__", "_");

                        // TODO: consider changing it
                        if key.is_empty() {
                            format!("{hash:x}")
                        } else {
                            format!("{hash:x}-{key}")
                        }
                    }

                    Ok(path.join(normalize_key(key)))
                })
            }),

            path_normalize: lua.create_function(|lua, path: LuaString| {
                let path = path.to_string_lossy()
                    .to_string();

                if path.is_empty() {
                    return Ok(LuaNil);
                }

                let (path, is_absolute) = match path.strip_prefix("/") {
                    Some(path) => (path, true),
                    None => (path.as_str(), false)
                };

                match split_path(path) {
                    Some(parts) => {
                        let mut path = parts.join("/");

                        if is_absolute {
                            path = format!("/{path}");
                        }

                        lua.create_string(path)
                            .map(LuaValue::String)
                    }

                    None if is_absolute => lua.create_string("/")
                        .map(LuaValue::String),

                    None => Ok(LuaNil)
                }
            })?,

            path_join: lua.create_function(|lua, parts: LuaMultiValue| {
                if parts.is_empty() {
                    return Ok(LuaNil);
                }

                let parts = parts.iter()
                    .flat_map(|part| part.to_string())
                    .filter(|part| !part.is_empty())
                    .collect::<Vec<_>>();

                let (parts, is_absolute) = match parts.first() {
                    None => return Ok(LuaNil),

                    Some(v) if v == "/" || v == "\\" => (&parts[1..], true),
                    Some(_) => (parts.as_slice(), false)
                };

                let Some(parts) = normalize_path_parts(parts) else {
                    if is_absolute {
                        return lua.create_string("/")
                            .map(LuaValue::String);
                    } else {
                        return Ok(LuaNil);
                    }
                };

                let mut path = parts.join("/");

                if is_absolute {
                    path = format!("/{path}");
                }

                lua.create_string(path)
                    .map(LuaValue::String)
            })?,

            path_parts: lua.create_function(|lua, path: LuaString| {
                let path = path.to_string_lossy()
                    .to_string();

                if path.is_empty() {
                    return Ok(LuaNil);
                }

                let path = path.strip_prefix("/")
                    .unwrap_or(&path);

                let Some(parts) = split_path(path) else {
                    return Ok(LuaNil);
                };

                let result = lua.create_table_with_capacity(parts.len(), 0)?;

                for part in parts {
                    result.raw_push(part)?;
                }

                Ok(LuaValue::Table(result))
            })?,

            path_parent: lua.create_function(|lua, path: LuaString| {
                let path = path.to_string_lossy()
                    .to_string();

                if path.is_empty() {
                    return Ok(LuaNil);
                }

                let (path, is_absolute) = match path.strip_prefix("/") {
                    Some(path) => (path, true),
                    None => (path.as_str(), false)
                };

                let Some(parts) = split_path(path) else {
                    return Ok(LuaNil);
                };

                if parts.len() > 1 {
                    let mut path = parts[..parts.len() - 1].join("/");

                    if is_absolute {
                        path = format!("/{path}");
                    }

                    lua.create_string(path)
                        .map(LuaValue::String)
                }

                else {
                    Ok(LuaNil)
                }
            })?,

            path_file_name: lua.create_function(|lua, path: LuaString| {
                let path = path.to_string_lossy()
                    .to_string();

                if path.is_empty() {
                    return Ok(LuaNil);
                }

                if path == "/" {
                    return lua.create_string("/")
                        .map(LuaValue::String);
                }

                let path = path.strip_prefix("/")
                    .unwrap_or(&path);

                let Some(mut parts) = split_path(path) else {
                    return Ok(LuaNil);
                };

                let Some(file_name) = parts.pop() else {
                    return Ok(LuaNil);
                };

                lua.create_string(file_name)
                    .map(LuaValue::String)
            })?,

            path_exists: Box::new(move |lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |_, mut path: PathBuf| {
                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    path = normalize_path(path, false)
                        .map_err(|err| {
                            LuaError::external(format!("failed to normalize path: {err}"))
                        })?;

                    Ok(path.exists())
                })
            }),

            path_permissions: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();

                lua.create_function(move |lua: &Lua, mut path: PathBuf| {
                    if path.is_relative() {
                        path = context.module_folder.join(path);
                    }

                    let result = lua.create_table_with_capacity(0, 2)?;

                    result.raw_set("read", context.can_read_path(&path)?)?;
                    result.raw_set("write", context.can_write_path(&path)?)?;

                    Ok(result)
                })
            }),

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 10)?;

        env.raw_set("temp_dir", (self.path_temp_dir)(&self.lua, context)?)?;
        env.raw_set("module_dir", (self.path_module_dir)(&self.lua, context)?)?;
        env.raw_set("persist_dir", (self.path_persist_dir)(&self.lua, context)?)?;
        env.raw_set("normalize", &self.path_normalize)?;
        env.raw_set("join", &self.path_join)?;
        env.raw_set("parts", &self.path_parts)?;
        env.raw_set("parent", &self.path_parent)?;
        env.raw_set("file_name", &self.path_file_name)?;
        env.raw_set("exists", (self.path_exists)(&self.lua, context)?)?;
        env.raw_set("permissions", (self.path_permissions)(&self.lua, context)?)?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_actions() -> Result<(), LuaError> {
        let api = PathApi::new(Lua::new())?;

        assert_eq!(api.path_normalize.call::<String>("/")?, "/");
        assert_eq!(api.path_normalize.call::<String>("a/b/c")?, "a/b/c");
        assert_eq!(api.path_normalize.call::<String>("/a/b/c")?, "/a/b/c");
        assert_eq!(api.path_normalize.call::<String>("a/./c")?, "a/c");
        assert_eq!(api.path_normalize.call::<String>("a/../c")?, "c");
        assert_eq!(api.path_normalize.call::<String>("a/../c/./")?, "c");
        assert_eq!(api.path_normalize.call::<String>("./a//\\./../b")?, "b");
        assert_eq!(api.path_normalize.call::<String>(" ")?, " "); // space is a correct entry name
        assert_eq!(api.path_normalize.call::<Option<String>>("")?, None); // entry name cannot be empty
        assert_eq!(api.path_normalize.call::<Option<String>>(".")?, None); // we do not support relative paths FIXME: resolve under module's folder?
        assert_eq!(api.path_normalize.call::<Option<String>>("..")?, None);
        assert_eq!(api.path_normalize.call::<Option<String>>("./..")?, None);
        assert_eq!(api.path_normalize.call::<Option<String>>("a/..")?, None);

        assert_eq!(api.path_join.call::<String>(("a", "b", "c"))?, "a/b/c");
        assert_eq!(api.path_join.call::<String>(("/", "a", "b", "c"))?, "/a/b/c");
        assert_eq!(api.path_join.call::<String>(("a", "..", "b"))?, "b");
        assert_eq!(api.path_join.call::<String>((".", "a", ".", "b"))?, "a/b");
        assert_eq!(api.path_join.call::<Option<String>>("")?, None);
        assert_eq!(api.path_join.call::<Option<String>>(".")?, None);
        assert_eq!(api.path_join.call::<Option<String>>("..")?, None);
        assert_eq!(api.path_join.call::<Option<String>>((".", ".."))?, None);
        assert_eq!(api.path_join.call::<Option<String>>(("a", ".."))?, None);

        assert_eq!(api.path_parts.call::<Vec<String>>("a/b/c")?, &["a", "b", "c"]);
        assert_eq!(api.path_parts.call::<Vec<String>>("a/./c")?, &["a", "c"]);
        assert_eq!(api.path_parts.call::<Vec<String>>("a/./c/..")?, &["a"]);
        assert_eq!(api.path_parts.call::<Vec<String>>("\\a/b/// /c")?, &["a", "b", " ", "c"]);
        assert_eq!(api.path_parts.call::<Option<Vec<String>>>("")?, None);
        assert_eq!(api.path_parts.call::<Option<Vec<String>>>(".")?, None);
        assert_eq!(api.path_parts.call::<Option<Vec<String>>>("..")?, None);
        assert_eq!(api.path_parts.call::<Option<Vec<String>>>("./..")?, None);
        assert_eq!(api.path_parts.call::<Option<Vec<String>>>("a/..")?, None);

        assert_eq!(api.path_parent.call::<String>("a/b/c")?, "a/b");
        assert_eq!(api.path_parent.call::<String>("/a/b/c")?, "/a/b");
        assert_eq!(api.path_parent.call::<String>("a\\./b")?, "a");
        assert_eq!(api.path_parent.call::<Option<Vec<String>>>("a")?, None);
        assert_eq!(api.path_parent.call::<Option<Vec<String>>>("a/.")?, None);
        assert_eq!(api.path_parent.call::<Option<Vec<String>>>("a/../b")?, None);

        assert_eq!(api.path_file_name.call::<String>("/")?, "/");
        assert_eq!(api.path_file_name.call::<String>("a")?, "a");
        assert_eq!(api.path_file_name.call::<String>("a/b/c")?, "c");
        assert_eq!(api.path_file_name.call::<String>("/a/b/c")?, "c");
        assert_eq!(api.path_file_name.call::<String>("a\\./b")?, "b");
        assert_eq!(api.path_file_name.call::<Option<Vec<String>>>(".")?, None);
        assert_eq!(api.path_file_name.call::<Option<Vec<String>>>("a/..")?, None);

        Ok(())
    }
}
