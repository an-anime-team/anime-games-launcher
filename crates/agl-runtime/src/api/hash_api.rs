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

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::io::Write;

use agl_core::tasks::fs::File;
use agl_core::tasks::io::{BufReader, AsyncReadExt};
use agl_core::hashes::{Hasher, HashAlgorithm};

use mlua::prelude::*;

use super::bytes::Bytes;
use super::task_api::{Promise, PromiseValue};
use super::*;

pub const HASHER_CHUNK_LEN: usize = 8 * 1024; // 8 KiB

pub struct HashApi {
    lua: Lua,

    hash_digitize: LuaFunction,
    hash_digitize_file: LuaFunctionBuilder,
    hash_hasher: LuaFunction,
    hash_write: LuaFunction,
    hash_finalize: LuaFunction
}

impl HashApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        let hasher_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            hash_digitize: lua.create_function(move |_, (algorithm, value): (LuaString, Bytes)| {
                let algorithm = HashAlgorithm::from_str(&algorithm.to_string_lossy())
                    .map_err(LuaError::external)?;

                let mut hasher = Hasher::new(algorithm);

                hasher.write_all(&value)?;
                hasher.flush()?;

                let hash = hasher.finalize().0;

                Ok(hash)
            })?,

            hash_digitize_file: {
                Box::new(move |lua: &Lua, context: &Context| {
                    let context = context.to_owned();

                    lua.create_function(move |lua: &Lua, (algorithm, mut path): (LuaString, PathBuf)| {
                        let algorithm = HashAlgorithm::from_str(&algorithm.to_string_lossy())
                            .map_err(LuaError::external)?;

                        if path.is_relative() {
                            path = context.module_folder.join(path);
                        }

                        path = normalize_path(path, true)
                            .map_err(|err| {
                                LuaError::external(format!("failed to normalize path: {err}"))
                            })?;

                        if !context.can_read_path(&path)? {
                            return Err(LuaError::external("no path read permissions"));
                        }

                        let value = PromiseValue::from_future(async move {
                            let mut file = BufReader::new(File::open(path).await?);
                            let mut hasher = Hasher::new(algorithm);

                            let mut buf = [0; HASHER_CHUNK_LEN];

                            loop {
                                let n = file.read(&mut buf).await?;

                                if n == 0 {
                                    break;
                                }

                                hasher.write_all(&buf[..n])?;
                                hasher.flush()?;
                            }

                            let hash = hasher.finalize().0;

                            Ok(Box::new(move |lua: &Lua| {
                                Bytes::new(hash)
                                    .into_lua(lua)
                            }) as task_api::TaskOutput)
                        });

                        Promise::new(value)
                            .into_lua(lua)
                    })
                })
            },

            hash_hasher: {
                let hasher_handles = hasher_handles.clone();

                lua.create_function(move |_, algorithm: LuaString| {
                    let algorithm = HashAlgorithm::from_str(&algorithm.to_string_lossy())
                        .map_err(LuaError::external)?;

                    let mut hashers = hasher_handles.lock()
                        .map_err(|err| {
                            LuaError::external("failed to register hasher handle")
                                .context(err)
                        })?;

                    let mut handle = rand::random::<i32>();

                    while hashers.contains_key(&handle) {
                        handle = rand::random::<i32>();
                    }

                    hashers.insert(handle, Hasher::new(algorithm));

                    Ok(handle)
                })?
            },

            hash_write: {
                let hasher_handles = hasher_handles.clone();

                lua.create_function(move |_, (handle, value): (i32, Bytes)| {
                    let mut hashers = hasher_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(hasher) = hashers.get_mut(&handle) else {
                        return Err(LuaError::external("invalid hasher handle"));
                    };

                    hasher.write_all(&value)?;
                    hasher.flush()?;

                    Ok(())
                })?
            },

            hash_finalize: {
                let hasher_handles = hasher_handles.clone();

                lua.create_function(move |lua: &Lua, handle: i32| {
                    let mut hashers = hasher_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(hasher) = hashers.remove(&handle) else {
                        return Err(LuaError::external("invalid hasher handle"));
                    };

                    Bytes::new(hasher.finalize().0)
                        .into_lua(lua)
                })?
            },

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 5)?;

        env.raw_set("digitize", &self.hash_digitize)?;
        env.raw_set("digitize_file", (self.hash_digitize_file)(&self.lua, context)?)?;
        env.raw_set("hasher", &self.hash_hasher)?;
        env.raw_set("write", &self.hash_write)?;
        env.raw_set("finalize", &self.hash_finalize)?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash() -> Result<(), LuaError> {
        let api = HashApi::new(Lua::new())?;

        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("seahash",  "Hello, World!"))?, &[46, 194, 87, 41, 102, 208, 6, 253]);
        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("crc32",    "Hello, World!"))?, &[236, 74, 195, 208]);
        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("crc32c",   "Hello, World!"))?, &[77, 85, 16, 104]);
        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("xxh32",    "Hello, World!"))?, &[64, 7, 222, 80]);
        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("xxh64",    "Hello, World!"))?, &[196, 154, 172, 248, 8, 15, 228, 127]);
        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("xxh3-64",  "Hello, World!"))?, &[96, 65, 93, 95, 97, 102, 2, 170]);
        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("xxh3-128", "Hello, World!"))?, &[83, 29, 242, 132, 68, 71, 221, 80, 119, 219, 3, 132, 44, 215, 83, 149]);
        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("md5",      "Hello, World!"))?, &[101, 168, 226, 125, 136, 121, 40, 56, 49, 182, 100, 189, 139, 127, 10, 212]);
        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("sha1",     "Hello, World!"))?, &[10, 10, 159, 42, 103, 114, 148, 37, 87, 171, 83, 85, 215, 106, 244, 66, 248, 246, 94, 1]);
        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("sha2-224", "Hello, World!"))?, &[114, 162, 61, 250, 65, 27, 166, 253, 224, 29, 191, 171, 243, 176, 10, 112, 156, 147, 235, 242, 115, 220, 41, 226, 216, 178, 97, 255]);
        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("sha2-256", "Hello, World!"))?, &[223, 253, 96, 33, 187, 43, 213, 176, 175, 103, 98, 144, 128, 158, 195, 165, 49, 145, 221, 129, 199, 247, 10, 75, 40, 104, 138, 54, 33, 130, 152, 111]);
        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("sha2-384", "Hello, World!"))?, &[84, 133, 204, 155, 51, 101, 180, 48, 93, 251, 78, 131, 55, 224, 165, 152, 165, 116, 248, 36, 43, 241, 114, 137, 224, 221, 108, 32, 163, 205, 68, 160, 137, 222, 22, 171, 74, 179, 8, 246, 62, 68, 177, 23, 14, 181, 245, 21]);
        assert_eq!(api.hash_digitize.call::<Vec<u8>>(("sha2-512", "Hello, World!"))?, &[55, 77, 121, 74, 149, 205, 207, 216, 179, 89, 147, 24, 95, 239, 155, 163, 104, 241, 96, 216, 218, 244, 50, 208, 139, 169, 241, 237, 30, 90, 190, 108, 198, 146, 145, 224, 250, 47, 224, 0, 106, 82, 87, 14, 241, 140, 25, 222, 244, 230, 23, 195, 60, 229, 46, 240, 166, 229, 251, 227, 24, 203, 3, 135]);

        Ok(())
    }

    #[test]
    fn hasher() -> Result<(), LuaError> {
        let api = HashApi::new(Lua::new())?;

        let hashers = [
            ("seahash",  vec![46, 194, 87, 41, 102, 208, 6, 253]),
            ("crc32",    vec![236, 74, 195, 208]),
            ("crc32c",   vec![77, 85, 16, 104]),
            ("xxh32",    vec![64, 7, 222, 80]),
            ("xxh64",    vec![196, 154, 172, 248, 8, 15, 228, 127]),
            ("xxh3-64",  vec![96, 65, 93, 95, 97, 102, 2, 170]),
            ("xxh3-128", vec![83, 29, 242, 132, 68, 71, 221, 80, 119, 219, 3, 132, 44, 215, 83, 149]),
            ("md5",      vec![101, 168, 226, 125, 136, 121, 40, 56, 49, 182, 100, 189, 139, 127, 10, 212]),
            ("sha1",     vec![10, 10, 159, 42, 103, 114, 148, 37, 87, 171, 83, 85, 215, 106, 244, 66, 248, 246, 94, 1]),
            ("sha2-224", vec![114, 162, 61, 250, 65, 27, 166, 253, 224, 29, 191, 171, 243, 176, 10, 112, 156, 147, 235, 242, 115, 220, 41, 226, 216, 178, 97, 255]),
            ("sha2-256", vec![223, 253, 96, 33, 187, 43, 213, 176, 175, 103, 98, 144, 128, 158, 195, 165, 49, 145, 221, 129, 199, 247, 10, 75, 40, 104, 138, 54, 33, 130, 152, 111]),
            ("sha2-384", vec![84, 133, 204, 155, 51, 101, 180, 48, 93, 251, 78, 131, 55, 224, 165, 152, 165, 116, 248, 36, 43, 241, 114, 137, 224, 221, 108, 32, 163, 205, 68, 160, 137, 222, 22, 171, 74, 179, 8, 246, 62, 68, 177, 23, 14, 181, 245, 21]),
            ("sha2-512", vec![55, 77, 121, 74, 149, 205, 207, 216, 179, 89, 147, 24, 95, 239, 155, 163, 104, 241, 96, 216, 218, 244, 50, 208, 139, 169, 241, 237, 30, 90, 190, 108, 198, 146, 145, 224, 250, 47, 224, 0, 106, 82, 87, 14, 241, 140, 25, 222, 244, 230, 23, 195, 60, 229, 46, 240, 166, 229, 251, 227, 24, 203, 3, 135])
        ];

        for (name, hash) in hashers {
            let hasher = api.hash_hasher.call::<i32>(name)?;

            api.hash_write.call::<()>((hasher, "Hello"))?;
            api.hash_write.call::<()>((hasher, ", "))?;
            api.hash_write.call::<()>((hasher, "World"))?;
            api.hash_write.call::<()>((hasher, "!"))?;

            assert_eq!(api.hash_finalize.call::<Bytes>(hasher)?.as_slice(), hash);
        }

        Ok(())
    }
}
