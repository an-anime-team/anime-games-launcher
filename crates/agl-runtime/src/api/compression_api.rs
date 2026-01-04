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

use std::str::FromStr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};

use agl_core::compression::{Compressor, Decompressor};

use mlua::prelude::*;

use super::bytes::Bytes;
use super::task_api::{Promise, PromiseValue, TaskOutput};

pub const COMPRESSOR_READ_CHUNK_SIZE: usize = 4096;

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

pub struct CompressionApi {
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

impl CompressionApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        let compression_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            compression_compress: lua.create_function(move |lua, (algorithm, value): (LuaString, Bytes)| {
                let mut compressor = Compressor::from_str(&algorithm.to_string_lossy())
                    .map_err(|err| {
                        LuaError::external("failed to create compressor")
                            .context(err)
                    })?;

                let len = value.len();

                let value = PromiseValue::from_blocking(move || {
                    compressor.write_all(&value)?;

                    drop(value);

                    compressor.flush()?;
                    compressor.try_finish()?;

                    let mut result = Vec::with_capacity(len);

                    compressor.read_to_end(&mut result)?;

                    Ok(Box::new(move |lua: &Lua| {
                        Bytes::new(result.into_boxed_slice())
                            .into_lua(lua)
                    }) as TaskOutput)
                });

                Promise::new(value)
                    .into_lua(lua)
            })?,

            compression_decompress: lua.create_function(move |lua, (algorithm, value): (LuaString, Bytes)| {
                let mut decompressor = Decompressor::from_str(&algorithm.to_string_lossy())
                    .map_err(|err| {
                        LuaError::external("failed to create decompressor")
                            .context(err)
                    })?;

                let len = value.len();

                let value = PromiseValue::from_blocking(move || {
                    decompressor.write_all(&value)?;

                    drop(value);

                    decompressor.flush()?;

                    let mut result = Vec::with_capacity(len);

                    decompressor.read_to_end(&mut result)?;

                    Ok(Box::new(move |lua: &Lua| {
                        Bytes::new(result.into_boxed_slice())
                            .into_lua(lua)
                    }) as TaskOutput)
                });

                Promise::new(value)
                    .into_lua(lua)
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

                    let mut buf = [0; COMPRESSOR_READ_CHUNK_SIZE];

                    let len = variant.read(&mut buf)?;

                    if len == 0 {
                        return Ok(LuaValue::Nil);
                    }

                    Bytes::new(buf[..len].to_vec().into_boxed_slice())
                        .into_lua(lua)
                })?
            },

            compression_write: {
                let compression_handles = compression_handles.clone();

                lua.create_function(move |_, (handle, value): (i32, Bytes)| {
                    let mut handles = compression_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to read compression handle")
                                .context(err)
                        })?;

                    let Some(variant) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid compression handle"));
                    };

                    variant.write_all(&value)?;
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

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 8)?;

        env.raw_set("compress", &self.compression_compress)?;
        env.raw_set("decompress", &self.compression_decompress)?;
        env.raw_set("compressor", &self.compression_compressor)?;
        env.raw_set("decompressor", &self.compression_decompressor)?;
        env.raw_set("read", &self.compression_read)?;
        env.raw_set("write", &self.compression_write)?;
        env.raw_set("finish", &self.compression_finish)?;
        env.raw_set("close", &self.compression_close)?;

        Ok(env)
    }
}
