use std::path::{Path, PathBuf};

use mlua::prelude::*;

use crate::core::prelude::*;
use crate::packages::prelude::*;
use crate::config;

use super::EngineError;

mod string_api;
mod path_api;
mod io_api;
mod network_api;
mod downloader_api;
mod archive_api;
mod hash_api;
mod sync_api;
mod process_api;

pub mod prelude {
    pub use super::string_api::StringAPI;
    pub use super::path_api::PathAPI;
    pub use super::io_api::IOAPI;
    pub use super::network_api::NetworkAPI;
    pub use super::downloader_api::DownloaderAPI;
    pub use super::archive_api::ArchiveAPI;
    pub use super::hash_api::HashAPI;
    pub use super::sync_api::SyncAPI;
    pub use super::process_api::ProcessAPI;

    pub use super::Standard;
}

use prelude::*;

lazy_static::lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("v1_runtime")
        .enable_all()
        .build()
        .expect("Failed to create v1 standard tasks runtime");
}

fn resolve_path(path: impl AsRef<str>) -> std::io::Result<PathBuf> {
    let mut path = PathBuf::from(path.as_ref());

    while path.is_symlink() {
        path = path.read_link()?;
    }

    Ok(path)
}

fn get_value_bytes(value: LuaValue) -> Result<Vec<u8>, LuaError> {
    match value {
        LuaValue::Number(value) => Ok(value.to_be_bytes().to_vec()),
        LuaValue::Integer(value) => Ok(value.to_be_bytes().to_vec()),
        LuaValue::String(value) => Ok(value.as_bytes().to_vec()),

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

fn slice_to_table(lua: &Lua, slice: impl AsRef<[u8]>) -> Result<LuaTable<'_>, LuaError> {
    let slice = slice.as_ref();
    let table = lua.create_table_with_capacity(slice.len(), 0)?;

    for byte in slice {
        table.push(*byte)?;
    }

    Ok(table)
}

pub struct Standard<'lua> {
    lua: &'lua Lua,

    clone: LuaFunction<'lua>,

    string_api: StringAPI<'lua>,
    path_api: PathAPI<'lua>,
    io_api: IOAPI<'lua>,
    network_api: NetworkAPI<'lua>,
    downloader_api: DownloaderAPI<'lua>,
    archive_api: ArchiveAPI<'lua>,
    hash_api: HashAPI<'lua>,
    sync_api: SyncAPI<'lua>,
    process_api: ProcessAPI<'lua>
}

impl<'lua> Standard<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, EngineError> {
        Ok(Self {
            lua,

            clone: lua.create_function(|lua, value: LuaValue| {
                fn clone_value<'lua>(lua: &'lua Lua, value: LuaValue<'lua>) -> Result<LuaValue<'lua>, LuaError> {
                    match value {
                        LuaValue::String(string) => {
                            Ok(LuaValue::String(lua.create_string(string.as_bytes())?))
                        }

                        LuaValue::Function(function) => {
                            Ok(LuaValue::Function(function.deep_clone()))
                        }

                        LuaValue::Table(table) => {
                            let cloned = lua.create_table_with_capacity(0, table.raw_len())?;

                            for pair in table.pairs::<LuaValue, LuaValue>() {
                                let (key, value) = pair?;

                                cloned.set(
                                    clone_value(lua, key)?,
                                    clone_value(lua, value)?
                                )?;
                            }

                            Ok(LuaValue::Table(cloned))
                        }

                        _ => Ok(value)
                    }
                }

                clone_value(lua, value)
            })?,

            string_api: StringAPI::new(lua)?,
            path_api: PathAPI::new(lua)?,
            io_api: IOAPI::new(lua)?,
            network_api: NetworkAPI::new(lua)?,
            downloader_api: DownloaderAPI::new(lua)?,
            archive_api: ArchiveAPI::new(lua)?,
            hash_api: HashAPI::new(lua)?,
            sync_api: SyncAPI::new(lua)?,
            process_api: ProcessAPI::new(lua)?
        })
    }

    /// Create new environment for the v1 modules standard.
    ///
    /// If `extended_privileges` enabled, then the result
    /// table will contain functions that can escape the
    /// default sandbox and execute code on the host machine.
    pub fn create_env(&self, extended_privileges: bool) -> Result<LuaTable<'lua>, EngineError> {
        let env = self.lua.create_table_with_capacity(0, if extended_privileges { 10 } else { 9 })?;

        env.set("clone", self.clone.clone())?;

        env.set("str", self.string_api.create_env()?)?;
        env.set("path", self.path_api.create_env()?)?;
        env.set("fs", self.io_api.create_env()?)?;
        env.set("net", self.network_api.create_env()?)?;
        env.set("downloader", self.downloader_api.create_env()?)?;
        env.set("archive", self.archive_api.create_env()?)?;
        env.set("hash", self.hash_api.create_env()?)?;
        env.set("sync", self.sync_api.create_env()?)?;

        // Extended privileges

        if extended_privileges {
            env.set("process", self.process_api.create_env()?)?;
        }

        Ok(env)
    }
}
