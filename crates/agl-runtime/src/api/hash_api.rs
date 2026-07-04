// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-runtime
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
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

pub const HASHER_CHUNK_LEN: usize = 8192; // 8 KiB hasher writes

pub struct HashApi {
    lua: Lua,

    hash_digitize: LuaFunction,
    hash_digitize_file: LuaFunctionBuilder,
    hash_hasher: LuaFunction,
    hash_write: LuaFunction,
    hash_finalize: LuaFunction
}

impl HashApi {
    pub fn new(lua: Lua, api_context: ApiContext) -> Result<Self, LuaError> {
        let hasher_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            hash_digitize: lua.create_function(move |lua: &Lua, (algorithm, value): (LuaString, Bytes)| {
                let algorithm = HashAlgorithm::from_str(&algorithm.to_string_lossy())
                    .map_err(LuaError::external)?;

                let mut hasher = Hasher::new(algorithm);

                hasher.write_all(&value)?;
                hasher.flush()?;

                let hash = hasher.finalize().0;

                Bytes::new(hash)
                    .into_lua(lua)
            })?,

            hash_digitize_file: {
                let api_context = api_context.clone();

                Box::new(move |lua: &Lua, module_context: &ModuleContext| {
                    let api_context = api_context.clone();
                    let module_context = module_context.clone();

                    lua.create_function(move |lua: &Lua, (algorithm, mut path): (LuaString, PathBuf)| {
                        let algorithm = HashAlgorithm::from_str(&algorithm.to_string_lossy())
                            .map_err(LuaError::external)?;

                        if path.is_relative() {
                            path = module_context.module_dir.join(path);
                        }

                        path = normalize_path(path, true)
                            .map_err(|err| {
                                LuaError::external(format!("failed to normalize path: {err}"))
                            })?;

                        if !api_context.can_access_path(&path) {
                            return Err(LuaError::external("this path cannot be accessed"));
                        }

                        if !module_context.can_read_path(&path) {
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

                lua.create_function(move |_lua: &Lua, algorithm: LuaString| {
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

                lua.create_function(move |_lua: &Lua, (handle, value): (i32, Bytes)| {
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
    pub fn create_env(
        &self,
        context: &ModuleContext
    ) -> Result<LuaTable, LuaError> {
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

    fn get_values() -> [(&'static str, Vec<u8>); 24] {
        [
            ("seahash",         vec![46, 194, 87, 41, 102, 208, 6, 253]),
            ("crc32",           vec![236, 74, 195, 208]),
            ("crc32c",          vec![77, 85, 16, 104]),
            ("siphash-1-3-64",  vec![41, 114, 161, 128, 56, 201, 39, 147]),
            ("siphash-1-3-128", vec![191, 107, 239, 175, 162, 213, 193, 236, 169, 129, 231, 119, 6, 126, 238, 15]),
            ("siphash-2-4-64",  vec![38, 73, 109, 52, 239, 10, 39, 31]),
            ("siphash-2-4-128", vec![50, 170, 110, 242, 8, 65, 163, 227, 18, 89, 105, 46, 109, 105, 177, 5]),
            ("xxh32",           vec![64, 7, 222, 80]),
            ("xxh64",           vec![196, 154, 172, 248, 8, 15, 228, 127]),
            ("xxh3-64",         vec![96, 65, 93, 95, 97, 102, 2, 170]),
            ("xxh3-128",        vec![83, 29, 242, 132, 68, 71, 221, 80, 119, 219, 3, 132, 44, 215, 83, 149]),
            ("md5",             vec![101, 168, 226, 125, 136, 121, 40, 56, 49, 182, 100, 189, 139, 127, 10, 212]),
            ("sha1",            vec![10, 10, 159, 42, 103, 114, 148, 37, 87, 171, 83, 85, 215, 106, 244, 66, 248, 246, 94, 1]),
            ("sha2-224",        vec![114, 162, 61, 250, 65, 27, 166, 253, 224, 29, 191, 171, 243, 176, 10, 112, 156, 147, 235, 242, 115, 220, 41, 226, 216, 178, 97, 255]),
            ("sha2-256",        vec![223, 253, 96, 33, 187, 43, 213, 176, 175, 103, 98, 144, 128, 158, 195, 165, 49, 145, 221, 129, 199, 247, 10, 75, 40, 104, 138, 54, 33, 130, 152, 111]),
            ("sha2-384",        vec![84, 133, 204, 155, 51, 101, 180, 48, 93, 251, 78, 131, 55, 224, 165, 152, 165, 116, 248, 36, 43, 241, 114, 137, 224, 221, 108, 32, 163, 205, 68, 160, 137, 222, 22, 171, 74, 179, 8, 246, 62, 68, 177, 23, 14, 181, 245, 21]),
            ("sha2-512",        vec![55, 77, 121, 74, 149, 205, 207, 216, 179, 89, 147, 24, 95, 239, 155, 163, 104, 241, 96, 216, 218, 244, 50, 208, 139, 169, 241, 237, 30, 90, 190, 108, 198, 146, 145, 224, 250, 47, 224, 0, 106, 82, 87, 14, 241, 140, 25, 222, 244, 230, 23, 195, 60, 229, 46, 240, 166, 229, 251, 227, 24, 203, 3, 135]),
            ("blake2s",         vec![236, 157, 185, 4, 214, 54, 239, 97, 241, 66, 27, 43, 164, 113, 18, 164, 250, 107, 137, 100, 253, 74, 10, 81, 72, 52, 69, 92, 33, 223, 120, 18]),
            ("blake2b",         vec![125, 253, 184, 136, 175, 113, 234, 224, 230, 166, 183, 81, 232, 227, 65, 61, 118, 126, 244, 250, 82, 167, 153, 61, 170, 158, 240, 151, 247, 170, 61, 148, 145, 153, 193, 19, 202, 163, 124, 148, 248, 12, 243, 178, 47, 125, 157, 110, 79, 93, 239, 79, 249, 39, 131, 12, 255, 228, 133, 124, 52, 190, 61, 137]),
            ("blake3",          vec![40, 138, 134, 167, 159, 32, 163, 214, 220, 205, 202, 119, 19, 190, 174, 209, 120, 121, 130, 150, 189, 250, 121, 19, 250, 42, 98, 217, 114, 123, 248, 248]),
            ("sha3-224",        vec![133, 48, 72, 251, 139, 17, 70, 43, 97, 0, 56, 86, 51, 192, 204, 141, 205, 198, 226, 184, 227, 118, 194, 129, 2, 188, 132, 242]),
            ("sha3-256",        vec![26, 241, 122, 102, 78, 63, 168, 228, 25, 184, 186, 5, 194, 161, 115, 22, 157, 247, 97, 98, 165, 162, 134, 224, 196, 5, 180, 96, 212, 120, 247, 239]),
            ("sha3-384",        vec![170, 154, 216, 164, 159, 49, 210, 221, 202, 187, 183, 1, 10, 21, 102, 65, 124, 255, 128, 63, 239, 80, 235, 162, 57, 85, 136, 38, 248, 114, 228, 104, 197, 116, 62, 127, 2, 107, 10, 142, 91, 45, 122, 28, 196, 101, 205, 190]),
            ("sha3-512",        vec![56, 224, 92, 51, 215, 176, 103, 18, 127, 33, 125, 140, 133, 110, 85, 79, 207, 240, 156, 147, 32, 184, 165, 151, 156, 226, 255, 93, 149, 221, 39, 186, 53, 209, 251, 165, 12, 86, 45, 253, 29, 108, 196, 139, 201, 197, 186, 164, 57, 8, 148, 65, 140, 201, 66, 217, 104, 249, 123, 203, 101, 148, 25, 237])
        ]
    }

    #[test]
    fn hash() -> Result<(), LuaError> {
        let api = HashApi::new(Lua::new(), ApiContext::default())?;

        for (algorithm, value) in get_values() {
            assert_eq!(
                api.hash_digitize.call::<LuaAnyUserData>((algorithm, "Hello, World!"))?
                    .call_method::<Vec<u8>>("as_table", ())?,
                value
            );
        }

        Ok(())
    }

    #[test]
    fn hasher() -> Result<(), LuaError> {
        let api = HashApi::new(Lua::new(), ApiContext::default())?;

        for (name, hash) in get_values() {
            let hasher = api.hash_hasher.call::<i32>(name)?;

            api.hash_write.call::<()>((hasher, "Hello"))?;
            api.hash_write.call::<()>((hasher, ", "))?;
            api.hash_write.call::<()>((hasher, "World"))?;
            api.hash_write.call::<()>((hasher, "!"))?;

            assert_eq!(
                api.hash_finalize.call::<LuaAnyUserData>(hasher)?
                    .call_method::<Vec<u8>>("as_table", ())?,
                hash
            );
        }

        Ok(())
    }
}
