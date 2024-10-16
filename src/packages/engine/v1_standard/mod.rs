use std::path::{Path, PathBuf};

use mlua::prelude::*;

use tokio::runtime::{
    Runtime,
    Builder as RuntimeBuilder
};

use crate::core::prelude::*;
use crate::packages::prelude::*;

use super::PackagesEngineError;

mod string_api;
mod path_api;
mod filesystem_api;
mod network_api;
mod downloader_api;
mod archive_api;
mod hash_api;
mod sync_api;
mod process_api;

pub use string_api::StringAPI;
pub use path_api::PathAPI;
pub use filesystem_api::FilesystemAPI;
pub use network_api::NetworkAPI;
pub use downloader_api::DownloaderAPI;
pub use archive_api::ArchiveAPI;
pub use hash_api::HashAPI;
pub use sync_api::SyncAPI;
pub use process_api::ProcessAPI;

lazy_static::lazy_static! {
    static ref RUNTIME: Runtime = RuntimeBuilder::new_multi_thread()
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

type LuaFunctionBuilder<'lua> = Box<dyn Fn(&'lua Lua, &Context) -> Result<LuaFunction<'lua>, LuaError>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Context {
    pub temp_folder: PathBuf,
    pub module_folder: PathBuf,
    pub persistent_folder: PathBuf,

    /// Include Process API in the environment.
    pub ext_process_api: bool
}

impl Context {
    /// Check if given path is accessible
    /// from the current context.
    pub fn is_accessible(&self, path: impl AsRef<Path>) -> bool {
        let allowed_paths = [
            &self.module_folder,
            &self.temp_folder,
            &self.persistent_folder
        ];

        let path = path.as_ref();

        for allowed_path in allowed_paths {
            if path.starts_with(allowed_path) {
                return true;
            }
        }

        false
    }
}

pub struct Standard<'lua> {
    lua: &'lua Lua,

    clone: LuaFunction<'lua>,

    string_api: StringAPI<'lua>,
    path_api: PathAPI<'lua>,
    filesystem_api: FilesystemAPI<'lua>,
    network_api: NetworkAPI<'lua>,
    downloader_api: DownloaderAPI<'lua>,
    archive_api: ArchiveAPI<'lua>,
    hash_api: HashAPI<'lua>,
    sync_api: SyncAPI<'lua>,
    process_api: ProcessAPI<'lua>
}

impl<'lua> Standard<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, PackagesEngineError> {
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
            filesystem_api: FilesystemAPI::new(lua)?,
            network_api: NetworkAPI::new(lua)?,
            downloader_api: DownloaderAPI::new(lua)?,
            archive_api: ArchiveAPI::new(lua)?,
            hash_api: HashAPI::new(lua)?,
            sync_api: SyncAPI::new(lua)?,
            process_api: ProcessAPI::new(lua)?
        })
    }

    /// Create new environment for the v1 modules standard
    /// using provided module context.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable<'lua>, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 10)?;

        env.set("clone", self.clone.clone())?;

        env.set("str", self.string_api.create_env()?)?;
        env.set("path", self.path_api.create_env(context)?)?;
        env.set("fs", self.filesystem_api.create_env(context)?)?;
        env.set("net", self.network_api.create_env()?)?;
        env.set("downloader", self.downloader_api.create_env(context)?)?;
        env.set("archive", self.archive_api.create_env(context)?)?;
        env.set("hash", self.hash_api.create_env()?)?;
        env.set("sync", self.sync_api.create_env()?)?;

        // Extended privileges

        if context.ext_process_api {
            env.set("process", self.process_api.create_env(context)?)?;
        }

        Ok(env)
    }
}
