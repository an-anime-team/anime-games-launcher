use std::collections::HashMap;
use std::fs::File;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};

use wineyard_core::hashes::{Hasher, HashAlgorithm};

use mlua::prelude::*;

use super::filesystem_api::IO_READ_CHUNK_LEN;
use super::*;

pub struct HashesAPI {
    lua: Lua,

    hashes_hash: LuaFunction,
    hashes_file_hash: LuaFunctionBuilder,
    hashes_hasher: LuaFunction,
    hashes_write: LuaFunction,
    hashes_finalize: LuaFunction
}

impl HashesAPI {
    pub fn new(lua: Lua) -> Result<Self, PackagesEngineError> {
        let hasher_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            hashes_hash: lua.create_function(move |_, (algorithm, value): (LuaString, LuaValue)| {
                let algorithm = HashAlgorithm::from_str(&algorithm.to_string_lossy())
                    .map_err(LuaError::external)?;

                let mut hasher = Hasher::new(algorithm);

                hasher.write_all(&lua_value_to_bytes(value)?)?;
                hasher.flush()?;

                let hash = hasher.finalize().0;

                Ok(hash)
            })?,

            hashes_file_hash: {
                Box::new(move |lua: &Lua, context: &Context| {
                    let context = context.to_owned();

                    lua.create_function(move |_, (algorithm, path): (LuaString, LuaString)| {
                        let algorithm = HashAlgorithm::from_str(&algorithm.to_string_lossy())
                            .map_err(LuaError::external)?;

                        let mut path = resolve_path(path.to_string_lossy())?;

                        if path.is_relative() {
                            path = context.module_folder.join(path);
                        }

                        if !context.is_accessible(&path) {
                            return Err(LuaError::external("path is inaccessible"));
                        }

                        let mut file = File::open(path)?;
                        let mut hasher = Hasher::new(algorithm);

                        let mut buf = [0; IO_READ_CHUNK_LEN];

                        loop {
                            let n = file.read(&mut buf)?;

                            if n == 0 {
                                break;
                            }

                            hasher.write_all(&buf[..n])?;
                            hasher.flush()?;
                        }

                        let hash = hasher.finalize().0;

                        Ok(hash)
                    })
                })
            },

            hashes_hasher: {
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

            hashes_write: {
                let hasher_handles = hasher_handles.clone();

                lua.create_function(move |_, (handle, value): (i32, LuaValue)| {
                    let mut hashers = hasher_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(hasher) = hashers.get_mut(&handle) else {
                        return Err(LuaError::external("invalid hasher handle"));
                    };

                    hasher.write_all(&lua_value_to_bytes(value)?)?;
                    hasher.flush()?;

                    Ok(())
                })?
            },

            hashes_finalize: {
                let hasher_handles = hasher_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    let mut hashers = hasher_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(hasher) = hashers.remove(&handle) else {
                        return Err(LuaError::external("invalid hasher handle"));
                    };

                    Ok(hasher.finalize().0)
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
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 5)?;

        env.raw_set("hash", self.hashes_hash.clone())?;
        env.raw_set("file_hash", (self.hashes_file_hash)(&self.lua, context)?)?;
        env.raw_set("hasher", self.hashes_hasher.clone())?;
        env.raw_set("write", self.hashes_write.clone())?;
        env.raw_set("finalize", self.hashes_finalize.clone())?;

        Ok(env)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn hash_calc() -> anyhow::Result<()> {
//         let api = HashAPI::new(Lua::new())?;

//         assert_eq!(api.hash_calc.call::<Vec<u8>>(0.5)?,             &[120, 18, 28, 179, 226, 204, 30, 109]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(-17)?,             &[176, 134, 88, 13, 238, 58, 194, 165]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>("Hello, World!")?, &[46, 194, 87, 41, 102, 208, 6, 253]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(vec![1, 2, 3])?,   &[142, 143, 212, 110, 1, 110, 210, 66]);

//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "seahash"))?,  &[46, 194, 87, 41, 102, 208, 6, 253]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "crc32"))?,    &[236, 74, 195, 208]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "crc32c"))?,   &[77, 85, 16, 104]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "xxh32"))?,    &[64, 7, 222, 80]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "xxh64"))?,    &[196, 154, 172, 248, 8, 15, 228, 127]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "xxh3-64"))?,  &[96, 65, 93, 95, 97, 102, 2, 170]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "xxh3-128"))?, &[83, 29, 242, 132, 68, 71, 221, 80, 119, 219, 3, 132, 44, 215, 83, 149]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "md5"))?,      &[101, 168, 226, 125, 136, 121, 40, 56, 49, 182, 100, 189, 139, 127, 10, 212]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "sha1"))?,     &[10, 10, 159, 42, 103, 114, 148, 37, 87, 171, 83, 85, 215, 106, 244, 66, 248, 246, 94, 1]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "sha2-224"))?, &[114, 162, 61, 250, 65, 27, 166, 253, 224, 29, 191, 171, 243, 176, 10, 112, 156, 147, 235, 242, 115, 220, 41, 226, 216, 178, 97, 255]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "sha2-256"))?, &[223, 253, 96, 33, 187, 43, 213, 176, 175, 103, 98, 144, 128, 158, 195, 165, 49, 145, 221, 129, 199, 247, 10, 75, 40, 104, 138, 54, 33, 130, 152, 111]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "sha2-384"))?, &[84, 133, 204, 155, 51, 101, 180, 48, 93, 251, 78, 131, 55, 224, 165, 152, 165, 116, 248, 36, 43, 241, 114, 137, 224, 221, 108, 32, 163, 205, 68, 160, 137, 222, 22, 171, 74, 179, 8, 246, 62, 68, 177, 23, 14, 181, 245, 21]);
//         assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "sha2-512"))?, &[55, 77, 121, 74, 149, 205, 207, 216, 179, 89, 147, 24, 95, 239, 155, 163, 104, 241, 96, 216, 218, 244, 50, 208, 139, 169, 241, 237, 30, 90, 190, 108, 198, 146, 145, 224, 250, 47, 224, 0, 106, 82, 87, 14, 241, 140, 25, 222, 244, 230, 23, 195, 60, 229, 46, 240, 166, 229, 251, 227, 24, 203, 3, 135]);

//         Ok(())
//     }

//     #[test]
//     fn hash_builder() -> anyhow::Result<()> {
//         let api = HashAPI::new(Lua::new())?;

//         let hashers = [
//             ("seahash",  vec![46, 194, 87, 41, 102, 208, 6, 253]),
//             ("crc32",    vec![236, 74, 195, 208]),
//             ("crc32c",   vec![77, 85, 16, 104]),
//             ("xxh32",    vec![64, 7, 222, 80]),
//             ("xxh64",    vec![196, 154, 172, 248, 8, 15, 228, 127]),
//             ("xxh3-64",  vec![96, 65, 93, 95, 97, 102, 2, 170]),
//             ("xxh3-128", vec![83, 29, 242, 132, 68, 71, 221, 80, 119, 219, 3, 132, 44, 215, 83, 149]),
//             ("md5",      vec![101, 168, 226, 125, 136, 121, 40, 56, 49, 182, 100, 189, 139, 127, 10, 212]),
//             ("sha1",     vec![10, 10, 159, 42, 103, 114, 148, 37, 87, 171, 83, 85, 215, 106, 244, 66, 248, 246, 94, 1]),
//             ("sha2-224", vec![114, 162, 61, 250, 65, 27, 166, 253, 224, 29, 191, 171, 243, 176, 10, 112, 156, 147, 235, 242, 115, 220, 41, 226, 216, 178, 97, 255]),
//             ("sha2-256", vec![223, 253, 96, 33, 187, 43, 213, 176, 175, 103, 98, 144, 128, 158, 195, 165, 49, 145, 221, 129, 199, 247, 10, 75, 40, 104, 138, 54, 33, 130, 152, 111]),
//             ("sha2-384", vec![84, 133, 204, 155, 51, 101, 180, 48, 93, 251, 78, 131, 55, 224, 165, 152, 165, 116, 248, 36, 43, 241, 114, 137, 224, 221, 108, 32, 163, 205, 68, 160, 137, 222, 22, 171, 74, 179, 8, 246, 62, 68, 177, 23, 14, 181, 245, 21]),
//             ("sha2-512", vec![55, 77, 121, 74, 149, 205, 207, 216, 179, 89, 147, 24, 95, 239, 155, 163, 104, 241, 96, 216, 218, 244, 50, 208, 139, 169, 241, 237, 30, 90, 190, 108, 198, 146, 145, 224, 250, 47, 224, 0, 106, 82, 87, 14, 241, 140, 25, 222, 244, 230, 23, 195, 60, 229, 46, 240, 166, 229, 251, 227, 24, 203, 3, 135])
//         ];

//         for (name, hash) in hashers {
//             let hasher = api.hash_builder.call::<i32>(name)?;

//             api.hash_write.call::<()>((hasher, "Hello"))?;
//             api.hash_write.call::<()>((hasher, ", "))?;
//             api.hash_write.call::<()>((hasher, "World"))?;
//             api.hash_write.call::<()>((hasher, "!"))?;

//             assert_eq!(api.hash_finalize.call::<Vec<u8>>(hasher)?, hash);
//         }

//         Ok(())
//     }
// }
