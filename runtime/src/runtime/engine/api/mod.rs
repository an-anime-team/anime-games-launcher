use std::path::{Path, PathBuf};

use wineyard_core::export::network::reqwest;

use mlua::prelude::*;
use mlua::Variadic;

use crate::hash::Hash;

use super::{PackagesEngineError, Options};

mod string_api;
mod path_api;
mod filesystem_api;
mod network_api;
mod downloader_api;
mod archives_api;
mod hashes_api;
mod compression_api;
mod sync_api;
mod sqlite_api;
// mod portals_api;
mod process_api;

pub use string_api::StringAPI;
pub use path_api::PathAPI;
pub use filesystem_api::FilesystemAPI;
pub use network_api::NetworkAPI;
pub use downloader_api::DownloaderAPI;
pub use archives_api::ArchivesAPI;
pub use hashes_api::HashesAPI;
pub use sync_api::SyncAPI;
pub use sqlite_api::SQLiteAPI;
// pub use portals_api::{
//     PortalsAPI,
//     Options as PortalsAPIOptions,
//     ToastOptions,
//     NotificationOptions,
//     DialogOptions,
//     DialogButton,
//     DialogButtonStatus
// };
pub use process_api::ProcessAPI;

/// Resolve symlink path into an actual filesystem entry path.
pub fn resolve_path(path: impl AsRef<str>) -> std::io::Result<PathBuf> {
    let mut path = PathBuf::from(path.as_ref());

    while path.is_symlink() {
        path = path.read_link()?;
    }

    Ok(path)
}

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
pub fn bytes_to_lua_table(lua: &Lua, slice: impl AsRef<[u8]>) -> Result<LuaTable, LuaError> {
    let slice = slice.as_ref();
    let table = lua.create_table_with_capacity(slice.len(), 0)?;

    for byte in slice {
        table.raw_push(*byte)?;
    }

    Ok(table)
}

type LuaFunctionBuilder = Box<dyn Fn(&Lua, &Context) -> Result<LuaFunction, LuaError>>;

#[derive(Debug, Clone)]
pub struct Context {
    pub resource_hash: Hash,

    pub temp_folder: PathBuf,
    pub module_folder: PathBuf,
    pub persistent_folder: PathBuf,
    pub input_resources: Vec<PathBuf>,

    /// Include Process API in the environment.
    pub ext_process_api: bool,

    /// Allow to access extra paths.
    pub ext_allowed_paths: Vec<PathBuf>,

    // pub local_validator: LocalValidator
}

impl Context {
    /// Check if given path is accessible from the current context.
    pub fn is_accessible(&self, path: impl AsRef<Path>) -> bool {
        let allowed_paths = [
            &self.module_folder,
            &self.temp_folder,
            &self.persistent_folder
        ];

        let allowed_paths = allowed_paths.into_iter()
            .chain(self.input_resources.iter())
            .chain(self.ext_allowed_paths.iter());

        let path = path.as_ref();

        for allowed_path in allowed_paths {
            if path.starts_with(allowed_path) {
                return true;
            }
        }

        // if let Some(allowed_paths) = self.local_validator.get_allowed_paths(&self.resource_hash) {
        //     for allowed_path in allowed_paths {
        //         if path.starts_with(allowed_path) {
        //             return true;
        //         }
        //     }
        // }

        false
    }
}

pub struct API {
    lua: Lua,

    clone: LuaFunction,
    dbg: LuaFunction,

    string_api: StringAPI,
    path_api: PathAPI,
    filesystem_api: FilesystemAPI,
    network_api: NetworkAPI,
    downloader_api: DownloaderAPI,
    archives_api: ArchivesAPI,
    hashes_api: HashesAPI,
    sync_api: SyncAPI,
    sqlite_api: SQLiteAPI,
    // portals_api: PortalsAPI,
    process_api: ProcessAPI
}

impl API {
    /// Create new v1 standard using provided lua engine.
    pub fn new(lua: Lua, _options: Options) -> Result<Self, PackagesEngineError> {
        let filesystem_api = FilesystemAPI::new(lua.clone())?;

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

            string_api: StringAPI::new(lua.clone())?,
            path_api: PathAPI::new(lua.clone())?,
            network_api: NetworkAPI::new(lua.clone(), reqwest::Client::new())?,
            downloader_api: DownloaderAPI::new(lua.clone())?,
            archives_api: ArchivesAPI::new(lua.clone())?,
            hashes_api: HashesAPI::new(lua.clone())?,
            sync_api: SyncAPI::new(lua.clone())?,
            sqlite_api: SQLiteAPI::new(lua.clone())?,
            // portals_api: PortalsAPI::new(lua.clone(), PortalsAPIOptions {
            //     show_toast: options.show_toast,
            //     show_notification: options.show_notification,
            //     show_dialog: options.show_dialog,
            //     file_handles: filesystem_api.file_handles()
            // })?,
            process_api: ProcessAPI::new(lua.clone())?,

            filesystem_api,
            lua
        })
    }

    #[inline(always)]
    pub const fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Create new environment for the API modules using provided context.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 12)?;

        env.set("clone", self.clone.clone())?;
        env.set("dbg", self.dbg.clone())?;

        env.set("str", self.string_api.create_env()?)?;
        env.set("path", self.path_api.create_env(context)?)?;
        env.set("fs", self.filesystem_api.create_env(context)?)?;
        env.set("net", self.network_api.create_env()?)?;
        env.set("downloader", self.downloader_api.create_env(context)?)?;
        env.set("archive", self.archives_api.create_env(context)?)?;
        env.set("hash", self.hashes_api.create_env(context)?)?;
        env.set("sync", self.sync_api.create_env()?)?;
        env.set("sqlite", self.sqlite_api.create_env(context)?)?;
        // env.set("portals", self.portals_api.create_env(context)?)?;

        // Extended privileges

        if context.ext_process_api {
            env.set("process", self.process_api.create_env(context)?)?;
        }

        Ok(env)
    }
}
