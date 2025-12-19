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

use std::path::{Path, PathBuf};

use agl_core::export::network::reqwest;

use mlua::prelude::*;
use mlua::Variadic;

// TODO: add tests.

mod string_api;
mod path_api;
mod filesystem_api;
mod network_api;
mod downloader_api;
mod archives_api;
mod hashes_api;
mod compression_api;
// mod sync_api;
mod sqlite_api;
// mod portals_api;
mod process_api;

use crate::module::ModuleScope;

/// Convert arbitrary lua value into bytes slice some reasonable way.
pub fn lua_value_to_bytes(value: LuaValue) -> Result<Vec<u8>, LuaError> {
    match value {
        LuaValue::Number(value)  => Ok(value.to_be_bytes().to_vec()),
        LuaValue::Integer(value) => Ok(value.to_be_bytes().to_vec()),
        LuaValue::String(value)  => Ok(value.as_bytes().to_vec()),

        // Assuming it's a vector of bytes.
        LuaValue::Table(table) => {
            let mut data = Vec::with_capacity(table.raw_len());

            for byte in table.sequence_values::<u8>() {
                data.push(byte?);
            }

            Ok(data)
        }

        _ => Err(LuaError::external("can't coerce given value to a bytes slice"))
    }
}

/// Make lua table from a bytes slice.
pub fn bytes_to_lua_table(
    lua: &Lua,
    slice: impl AsRef<[u8]>
) -> Result<LuaTable, LuaError> {
    let slice = slice.as_ref();
    let table = lua.create_table_with_capacity(slice.len(), 0)?;

    for byte in slice {
        table.raw_push(*byte)?;
    }

    Ok(table)
}

/// Normalize path by resolving symbolic links.
pub fn normalize_path(mut path: PathBuf) -> std::io::Result<PathBuf> {
    while path.is_symlink() {
        path = path.read_link()?;
    }

    Ok(path.components().collect())
}

/// Luau module standard library builder context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    /// Path to a temporary folder. It is always accessible for the module.
    pub temp_folder: PathBuf,

    /// Path to a module's personal folder. It is always accessible for the
    /// module.
    pub module_folder: PathBuf,

    /// Path to a inter-modules globally accessible folder.
    pub persistent_folder: PathBuf,

    /// Module permissions scope.
    pub scope: ModuleScope
}

impl Context {
    /// Check if a path is allowed to be accessed by the current module.
    pub fn is_accessible(
        &self,
        path: impl Into<PathBuf>
    ) -> std::io::Result<bool> {
        fn is_parent_of(parent: &Path, child: &Path) -> bool {
            parent.components()
                .zip(child.components())
                .all(|(p, c)| p == c)
        }

        let path = normalize_path(path.into())?;

        if is_parent_of(&self.temp_folder, &path)
            || is_parent_of(&self.module_folder, &path)
            || is_parent_of(&self.persistent_folder, &path)
        {
            return Ok(true);
        }

        for allowed_path in &self.scope.sandbox_allowed_paths {
            if is_parent_of(allowed_path, &path) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

type LuaFunctionBuilder = Box<dyn Fn(&Lua, &Context) -> Result<LuaFunction, LuaError>>;

/// Luau modules standard library builder.
pub struct Api {
    lua: Lua,

    clone: LuaFunction,
    dbg: LuaFunction,

    string_api: string_api::StringApi,
    path_api: path_api::PathApi,
    filesystem_api: filesystem_api::FilesystemApi,
    network_api: network_api::NetworkApi,
    downloader_api: downloader_api::DownloaderApi,
    archives_api: archives_api::ArchivesApi,
    hashes_api: hashes_api::HashesApi,
    compression_api: compression_api::CompressionApi,
    // sync_api: sync_api::SyncApi,
    sqlite_api: sqlite_api::SqliteApi,
    // portals_api: PortalsAPI,
    process_api: process_api::ProcessApi
}

impl Api {
    /// Create new standard library builder.
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        Ok(Self {
            clone: lua.create_function(|lua, value: LuaValue| {
                fn clone_value(lua: &Lua, value: LuaValue) -> Result<LuaValue, LuaError> {
                    match value {
                        LuaValue::String(string) => {
                            Ok(LuaValue::String(lua.create_string(string.as_bytes())?))
                        }

                        LuaValue::Function(function) => {
                            Ok(LuaValue::Function(function.deep_clone()))
                        }

                        LuaValue::Table(table) => {
                            let cloned = lua.create_table_with_capacity(0, table.raw_len())?;

                            table.for_each(|key, value| {
                                cloned.raw_set(
                                    clone_value(lua, key)?,
                                    clone_value(lua, value)?
                                )
                            })?;

                            cloned.set_metatable(table.metatable());

                            Ok(LuaValue::Table(cloned))
                        }

                        _ => Ok(value)
                    }
                }

                clone_value(lua, value)
            })?,

            dbg: lua.create_function(|_, values: Variadic<LuaValue>| {
                for value in values {
                    #[cfg(feature = "tracing")]
                    tracing::debug!("{value:#?}");

                    #[cfg(not(feature = "tracing"))]
                    dbg!(value);
                }

                Ok(())
            })?,

            string_api: string_api::StringApi::new(lua.clone())?,
            path_api: path_api::PathApi::new(lua.clone())?,
            filesystem_api: filesystem_api::FilesystemApi::new(lua.clone())?,
            network_api: network_api::NetworkApi::new(lua.clone(), reqwest::Client::new())?, // TODO: propagate proxy and timeout values
            downloader_api: downloader_api::DownloaderApi::new(lua.clone())?,
            archives_api: archives_api::ArchivesApi::new(lua.clone())?,
            hashes_api: hashes_api::HashesApi::new(lua.clone())?,
            compression_api: compression_api::CompressionApi::new(lua.clone())?,
            // sync_api: sync_api::SyncApi::new(lua.clone())?,
            sqlite_api: sqlite_api::SqliteApi::new(lua.clone())?,
            // portals_api: PortalsAPI::new(lua.clone(), PortalsAPIOptions {
            //     show_toast: options.show_toast,
            //     show_notification: options.show_notification,
            //     show_dialog: options.show_dialog,
            //     file_handles: filesystem_api.file_handles()
            // })?,
            process_api: process_api::ProcessApi::new(lua.clone())?,

            lua
        })
    }

    #[inline(always)]
    pub const fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Create new environment table.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table()?;

        // Some default functions and constants.
        let versions_table = self.lua.create_table_with_capacity(0, 2)?;

        versions_table.raw_set("core", agl_core::VERSION)?;
        versions_table.raw_set("runtime", crate::VERSION)?;

        env.raw_set("versions", versions_table)?;
        env.raw_set("clone", self.clone.clone())?;
        env.raw_set("dbg", self.dbg.clone())?;

        // Some default lua functions.
        env.raw_set("table", self.lua.globals().get::<LuaValue>("table")?)?;
        env.raw_set("string", self.lua.globals().get::<LuaValue>("string")?)?;
        env.raw_set("math", self.lua.globals().get::<LuaValue>("math")?)?;
        env.raw_set("coroutine", self.lua.globals().get::<LuaValue>("coroutine")?)?;

        // String API.
        if context.scope.allow_string_api {
            env.raw_set("str", self.string_api.create_env()?)?;
        }

        // Path API.
        if context.scope.allow_path_api {
            env.raw_set("path", self.path_api.create_env(context)?)?;
        }

        // Filesystem API.
        if context.scope.allow_basic_fs_api {
            env.raw_set("fs", self.filesystem_api.create_env(context)?)?;
        }

        // Network API.
        if context.scope.allow_network_api {
            env.raw_set("net", self.network_api.create_env()?)?;
        }

        // Downloader API.
        if context.scope.allow_downloader_api {
            env.raw_set("downloader", self.downloader_api.create_env(context)?)?;
        }

        // Archives API.
        if context.scope.allow_archives_api {
            env.raw_set("archive", self.archives_api.create_env(context)?)?;
        }

        // Hashes API.
        if context.scope.allow_hashes_api {
            env.raw_set("hash", self.hashes_api.create_env(context)?)?;
        }

        // Compression API.
        if context.scope.allow_compression_api {
            env.raw_set("compression", self.compression_api.create_env()?)?;
        }

        // Sqlite API.
        if context.scope.allow_sqlite_api {
            env.raw_set("sqlite", self.sqlite_api.create_env(context)?)?;
        }

        // Process API.
        if context.scope.allow_process_api {
            env.raw_set("process", self.process_api.create_env(context)?)?;
        }

        Ok(env)
    }
}
