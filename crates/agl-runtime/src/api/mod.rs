// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-runtime
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@vk.com>
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
use std::sync::{Arc, RwLock};

use mlua::prelude::*;

use agl_core::export::network::reqwest;
use agl_core::tasks;

// TODO: add tests.

pub mod bytes;

pub mod string_api;
pub mod path_api;
pub mod task_api;
pub mod system_api;
pub mod filesystem_api;
pub mod http_api;
pub mod downloader_api;
pub mod archive_api;
pub mod hash_api;
pub mod compression_api;
pub mod sqlite_api;
pub mod torrent_api;
pub mod portal_api;
pub mod process_api;

use crate::module::ModuleScope;

/// Normalize path by resolving symbolic links.
pub fn normalize_path(
    mut path: PathBuf,
    resolve_symlinks: bool
) -> std::io::Result<PathBuf> {
    while resolve_symlinks && path.is_symlink() {
        path = path.read_link()?;
    }

    Ok(path.components().collect())
}

fn path_is_parent_of(parent: &Path, child: &Path) -> bool {
    parent.components()
        .zip(child.components())
        .all(|(p, c)| p == c)
}

/// Luau module standard library builder context.
#[derive(Debug, Clone)]
pub struct Context {
    /// Path to a temporary folder. It is always accessible for the module.
    pub temp_folder: PathBuf,

    /// Path to a module's personal folder. It is always accessible for the
    /// module.
    pub module_folder: PathBuf,

    /// Path to a inter-modules globally accessible folder.
    pub persistent_folder: PathBuf,

    /// Module permissions scope.
    pub scope: Arc<RwLock<ModuleScope>>
}

impl Context {
    /// Check if a path is allowed to be read by the current module.
    pub fn can_read_path(
        &self,
        path: &Path
    ) -> std::io::Result<bool> {
        if path_is_parent_of(&self.temp_folder, path)
            || path_is_parent_of(&self.module_folder, path)
            || path_is_parent_of(&self.persistent_folder, path)
        {
            return Ok(true);
        }

        if let Ok(scope) = self.scope.read() {
            let rw_paths = scope.sandbox_read_paths.iter()
                .chain(scope.sandbox_write_paths.iter());

            for allowed_path in rw_paths {
                if path_is_parent_of(allowed_path, path) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Check if a path is allowed to be written by the current module.
    pub fn can_write_path(
        &self,
        path: &Path
    ) -> std::io::Result<bool> {
        if path_is_parent_of(&self.temp_folder, path)
            || path_is_parent_of(&self.module_folder, path)
            || path_is_parent_of(&self.persistent_folder, path)
        {
            return Ok(true);
        }

        if let Ok(scope) = self.scope.read() {
            for allowed_path in &scope.sandbox_write_paths {
                if path_is_parent_of(allowed_path, path) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

type LuaFunctionBuilder = Box<dyn Fn(&Lua, &Context) -> Result<LuaFunction, LuaError>>;

pub struct ApiOptions {
    /// Lua engine.
    pub lua: Lua,

    /// Reqwest client used by the network API.
    pub reqwest_client: reqwest::Client,

    /// BitTorrent server instance. If `None` is provided then the torrent API
    /// will be disabled for all the modules.
    pub torrent_server: Option<torrent_api::TorrentServer>,

    /// Callback used to display a toast message.
    pub show_toast: Box<dyn Fn(portal_api::ToastOptions) + Send>,

    /// Callback used to display a system notification.
    pub show_notification: Box<dyn Fn(portal_api::NotificationOptions) + Send>,

    /// Callback used to display a dialog.
    pub show_dialog: Box<dyn Fn(portal_api::DialogOptions) + Send>,

    /// Callback used to translate localizable string.
    pub translate: fn(agl_locale::string::LocalizableString) -> String
}

/// Luau modules standard library builder.
pub struct Api {
    lua: Lua,

    clone: LuaFunction,
    dbg: LuaFunction,
    sleep: LuaFunction,
    r#await: LuaFunction,

    string_api: string_api::StringApi,
    path_api: path_api::PathApi,
    task_api: task_api::TaskApi,
    system_api: system_api::SystemApi,
    filesystem_api: filesystem_api::FilesystemApi,
    http_api: http_api::HttpApi,
    downloader_api: downloader_api::DownloaderApi,
    archive_api: archive_api::ArchiveApi,
    hash_api: hash_api::HashApi,
    compression_api: compression_api::CompressionApi,
    sqlite_api: sqlite_api::SqliteApi,
    torrent_api: Option<torrent_api::TorrentApi>,
    portal_api: portal_api::PortalApi,
    process_api: process_api::ProcessApi
}

impl Api {
    /// Create new standard library builder.
    pub fn new(options: ApiOptions) -> Result<Self, LuaError> {
        Ok(Self {
            clone: options.lua.create_function(|lua, value: LuaValue| {
                fn clone_value(lua: &Lua, value: LuaValue) -> Result<LuaValue, LuaError> {
                    match value {
                        LuaValue::String(string) => {
                            Ok(LuaValue::String(lua.create_string(string.as_bytes())?))
                        }

                        LuaValue::Function(function) => {
                            Ok(LuaValue::Function(function.deep_clone()?))
                        }

                        LuaValue::Table(table) => {
                            let cloned = lua.create_table_with_capacity(0, table.raw_len())?;

                            table.for_each(|key, value| {
                                cloned.raw_set(
                                    clone_value(lua, key)?,
                                    clone_value(lua, value)?
                                )
                            })?;

                            cloned.set_metatable(table.metatable())?;

                            Ok(LuaValue::Table(cloned))
                        }

                        _ => Ok(value)
                    }
                }

                clone_value(lua, value)
            })?,

            dbg: options.lua.create_function(|_, values: LuaVariadic<LuaValue>| {
                for value in values {
                    #[cfg(feature = "tracing")]
                    tracing::debug!("{value:#?}");

                    #[cfg(not(feature = "tracing"))]
                    dbg!(value);
                }

                Ok(())
            })?,

            sleep: options.lua.create_function(|_, (duration, callback): (u32, Option<LuaFunction>)| {
                let duration = std::time::Duration::from_millis(duration as u64);

                if let Some(callback) = callback {
                    tasks::spawn(async move {
                        tasks::sleep(duration).await;

                        #[allow(unused)]
                        if let Err(err) = callback.call::<()>(()) {
                            #[cfg(feature = "tracing")]
                            tracing::error!(?err, "sleep callback execution error");
                        }
                    });
                }

                else {
                    std::thread::sleep(duration);
                }

                Ok(())
            })?,

            r#await: options.lua.create_function(|_, task: LuaValue| -> Result<LuaValue, LuaError> {
                match task {
                    LuaValue::Thread(coroutine) => {
                        tasks::block_on(coroutine.into_async::<LuaValue>(())?)
                    }

                    LuaValue::Function(callback) => {
                        callback.call(())
                    }

                    LuaValue::UserData(object) if object.type_name()?.as_deref() == Some("Promise") => {
                        object.call_method::<LuaValue>("await", ())
                    }

                    _ => Ok(task)
                }
            })?,

            string_api: string_api::StringApi::new(options.lua.clone())?,
            path_api: path_api::PathApi::new(options.lua.clone())?,
            task_api: task_api::TaskApi::new(options.lua.clone())?,
            system_api: system_api::SystemApi::new(options.lua.clone())?,
            filesystem_api: filesystem_api::FilesystemApi::new(options.lua.clone())?,
            http_api: http_api::HttpApi::new(options.lua.clone(), options.reqwest_client.clone())?,
            downloader_api: downloader_api::DownloaderApi::new(options.lua.clone(), options.reqwest_client.clone())?,
            archive_api: archive_api::ArchiveApi::new(options.lua.clone())?,
            hash_api: hash_api::HashApi::new(options.lua.clone())?,
            compression_api: compression_api::CompressionApi::new(options.lua.clone())?,
            sqlite_api: sqlite_api::SqliteApi::new(options.lua.clone())?,
            torrent_api: options.torrent_server.map(|server| {
                torrent_api::TorrentApi::new(options.lua.clone(), server)
            }).transpose()?,
            portal_api: portal_api::PortalApi::new(options.lua.clone(), portal_api::PortalApiOptions {
                show_toast: options.show_toast,
                show_notification: options.show_notification,
                show_dialog: options.show_dialog,
                translate: options.translate
            })?,
            process_api: process_api::ProcessApi::new(options.lua.clone())?,

            lua: options.lua
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

        env.raw_set("clone", &self.clone)?;
        env.raw_set("dbg", &self.dbg)?;
        env.raw_set("sleep", &self.sleep)?;
        env.raw_set("await", &self.r#await)?;

        // Some default lua functions.
        env.raw_set("print", self.lua.globals().get::<LuaFunction>("print")?)?;

        env.raw_set("pairs", self.lua.globals().get::<LuaFunction>("pairs")?)?;
        env.raw_set("ipairs", self.lua.globals().get::<LuaFunction>("ipairs")?)?;
        env.raw_set("next", self.lua.globals().get::<LuaFunction>("next")?)?;

        env.raw_set("assert", self.lua.globals().get::<LuaFunction>("assert")?)?;
        env.raw_set("error", self.lua.globals().get::<LuaFunction>("error")?)?;
        env.raw_set("pcall", self.lua.globals().get::<LuaFunction>("pcall")?)?;
        env.raw_set("xpcall", self.lua.globals().get::<LuaFunction>("xpcall")?)?;

        env.raw_set("tonumber", self.lua.globals().get::<LuaFunction>("tonumber")?)?;
        env.raw_set("tostring", self.lua.globals().get::<LuaFunction>("tostring")?)?;
        env.raw_set("type", self.lua.globals().get::<LuaFunction>("type")?)?;
        env.raw_set("typeof", self.lua.globals().get::<LuaFunction>("typeof")?)?;

        env.raw_set("getmetatable", self.lua.globals().get::<LuaFunction>("getmetatable")?)?;
        env.raw_set("setmetatable", self.lua.globals().get::<LuaFunction>("setmetatable")?)?;
        env.raw_set("rawget", self.lua.globals().get::<LuaFunction>("rawget")?)?;
        env.raw_set("rawset", self.lua.globals().get::<LuaFunction>("rawset")?)?;

        env.raw_set("table", self.lua.globals().get::<LuaValue>("table")?)?;
        env.raw_set("string", self.lua.globals().get::<LuaValue>("string")?)?;
        env.raw_set("math", self.lua.globals().get::<LuaValue>("math")?)?;
        env.raw_set("coroutine", self.lua.globals().get::<LuaValue>("coroutine")?)?;

        let Ok(scope) = context.scope.read() else {
            return Err(LuaError::external("failed to lock module scope"));
        };

        // String API.
        if scope.allow_string_api {
            env.raw_set("str", self.string_api.create_env()?)?;
        }

        // Path API.
        if scope.allow_path_api {
            env.raw_set("path", self.path_api.create_env(context)?)?;
        }

        // Task API.
        if scope.allow_task_api {
            env.raw_set("task", self.task_api.create_env()?)?;
        }

        // System API.
        if scope.allow_system_api {
            env.raw_set("system", self.system_api.create_env()?)?;
        }

        // Filesystem API.
        if scope.allow_filesystem_api {
            env.raw_set("fs", self.filesystem_api.create_env(context)?)?;
        }

        // HTTP API.
        if scope.allow_http_api {
            env.raw_set("http", self.http_api.create_env()?)?;
        }

        // Downloader API.
        if scope.allow_downloader_api {
            env.raw_set("downloader", self.downloader_api.create_env(context)?)?;
        }

        // Archive API.
        if scope.allow_archive_api {
            env.raw_set("archive", self.archive_api.create_env(context)?)?;
        }

        // Hash API.
        if scope.allow_hash_api {
            env.raw_set("hash", self.hash_api.create_env(context)?)?;
        }

        // Compression API.
        if scope.allow_compression_api {
            env.raw_set("compression", self.compression_api.create_env()?)?;
        }

        // Sqlite API.
        if scope.allow_sqlite_api {
            env.raw_set("sqlite", self.sqlite_api.create_env(context)?)?;
        }

        // Torrent API.
        if let Some(torrent_api) = &self.torrent_api && scope.allow_torrent_api {
            env.raw_set("torrent", torrent_api.create_env(context)?)?;
        }

        // Portal API.
        if scope.allow_portal_api {
            env.raw_set("portal", self.portal_api.create_env(context)?)?;
        }

        // Process API.
        if scope.allow_process_api {
            env.raw_set("process", self.process_api.create_env(context)?)?;
        }

        Ok(env)
    }
}
