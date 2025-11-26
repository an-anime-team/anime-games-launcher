use std::str::FromStr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};

use wineyard_core::compression::{Compressor, Decompressor};

use mlua::prelude::*;

use super::filesystem_api::IO_READ_CHUNK_LEN;
use super::*;

enum Variant {
    Compressor(Compressor),
    Decompressor(Decompressor)
}

impl Read for Variant {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Compressor(compressor) => compressor.read(buf),
            Self::Decompressor(decompressor) => decompressor.read(buf)
        }
    }
}

impl Write for Variant {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Compressor(compressor) => compressor.write(buf),
            Self::Decompressor(decompressor) => decompressor.write(buf)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Compressor(compressor) => compressor.flush(),
            Self::Decompressor(decompressor) => decompressor.flush()
        }
    }
}

pub struct CompressionAPI {
    lua: Lua,

    compression_compress: LuaFunction,
    compression_decompress: LuaFunction,
    compression_compressor: LuaFunction,
    compression_decompressor: LuaFunction,
    compression_read: LuaFunction,
    compression_write: LuaFunction,
    compression_finish: LuaFunction,
    compression_close: LuaFunction
}

impl CompressionAPI {
    pub fn new(lua: Lua) -> Result<Self, PackagesEngineError> {
        let compression_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            compression_compress: lua.create_function(move |lua, (algorithm, value): (LuaString, LuaValue)| {
                let mut compressor = Compressor::from_str(&algorithm.to_string_lossy())
                    .map_err(|err| {
                        LuaError::external("failed to create compressor")
                            .context(err)
                    })?;

                let value = lua_value_to_bytes(value)?;
                let len = value.len();

                compressor.write_all(&value)?;

                drop(value);

                compressor.flush()?;
                compressor.try_finish()?;

                let mut result = Vec::with_capacity(len);

                compressor.read_to_end(&mut result)?;

                bytes_to_lua_table(lua, result)
            })?,

            compression_decompress: lua.create_function(move |lua, (algorithm, value): (LuaString, LuaValue)| {
                let mut decompressor = Decompressor::from_str(&algorithm.to_string_lossy())
                    .map_err(|err| {
                        LuaError::external("failed to create decompressor")
                            .context(err)
                    })?;

                let value = lua_value_to_bytes(value)?;
                let len = value.len();

                decompressor.write_all(&value)?;

                drop(value);

                decompressor.flush()?;

                let mut result = Vec::with_capacity(len);

                decompressor.read_to_end(&mut result)?;

                bytes_to_lua_table(lua, result)
            })?,

            compression_compressor: {
                let compression_handles = compression_handles.clone();

                lua.create_function(move |_, algorithm: LuaString| {
                    let compressor = Compressor::from_str(&algorithm.to_string_lossy())
                        .map_err(|err| {
                            LuaError::external("failed to create compressor")
                                .context(err)
                        })?;

                    let mut handles = compression_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to register compressor handle")
                                .context(err)
                        })?;

                    let mut handle = rand::random::<i32>();

                    while handles.contains_key(&handle) {
                        handle = rand::random::<i32>();
                    }

                    handles.insert(handle, Variant::Compressor(compressor));

                    Ok(handle)
                })?
            },

            compression_decompressor: {
                let compression_handles = compression_handles.clone();

                lua.create_function(move |_, algorithm: LuaString| {
                    let decompressor = Decompressor::from_str(&algorithm.to_string_lossy())
                        .map_err(|err| {
                            LuaError::external("failed to create decompressor")
                                .context(err)
                        })?;

                    let mut handles = compression_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to register decompressor handle")
                                .context(err)
                        })?;

                    let mut handle = rand::random::<i32>();

                    while handles.contains_key(&handle) {
                        handle = rand::random::<i32>();
                    }

                    handles.insert(handle, Variant::Decompressor(decompressor));

                    Ok(handle)
                })?
            },

            compression_read: {
                let compression_handles = compression_handles.clone();

                lua.create_function(move |lua, handle: i32| {
                    let mut handles = compression_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to read compression handle")
                                .context(err)
                        })?;

                    let Some(variant) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid compression handle"));
                    };

                    let mut buf = [0; IO_READ_CHUNK_LEN];

                    let len = variant.read(&mut buf)?;

                    bytes_to_lua_table(lua, &buf[..len])
                })?
            },

            compression_write: {
                let compression_handles = compression_handles.clone();

                lua.create_function(move |_, (handle, value): (i32, LuaValue)| {
                    let mut handles = compression_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to read compression handle")
                                .context(err)
                        })?;

                    let Some(variant) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid compression handle"));
                    };

                    variant.write_all(&lua_value_to_bytes(value)?)?;
                    variant.flush()?;

                    Ok(())
                })?
            },

            compression_finish: {
                let compression_handles = compression_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    let mut handles = compression_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to read compression handle")
                                .context(err)
                        })?;

                    let Some(variant) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid compression handle"));
                    };

                    if let Variant::Compressor(compressor) = variant {
                        compressor.try_finish()?;
                    }

                    Ok(())
                })?
            },

            compression_close: {
                let compression_handles = compression_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    compression_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to read compression handle")
                                .context(err)
                        })?
                        .remove(&handle);

                    Ok(())
                })?
            },

            lua
        })
    }

    #[inline(always)]
    pub const fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 8)?;

        env.raw_set("compress", self.compression_compress.clone())?;
        env.raw_set("decompress", self.compression_decompress.clone())?;
        env.raw_set("compressor", self.compression_compressor.clone())?;
        env.raw_set("decompressor", self.compression_decompressor.clone())?;
        env.raw_set("read", self.compression_read.clone())?;
        env.raw_set("write", self.compression_write.clone())?;
        env.raw_set("finish", self.compression_finish.clone())?;
        env.raw_set("close", self.compression_close.clone())?;

        Ok(env)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

// }
