use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::Write;

use mlua::prelude::*;

use super::*;

enum Hasher {
    Seahash(seahash::SeaHasher),
    Crc32(crc32fast::Hasher),
    Crc32c(crc32c::Crc32cHasher),
    Xxh32(xxhash_rust::xxh32::Xxh32),
    Xxh64(xxhash_rust::xxh64::Xxh64),
    Xxh3_64(xxhash_rust::xxh3::Xxh3),
    Xxh3_128(xxhash_rust::xxh3::Xxh3),
    Md5(md5::Context),
    Sha1(sha1::Sha1),
    Sha2_224(sha2::Sha224),
    Sha2_256(sha2::Sha256),
    Sha2_384(sha2::Sha384),
    Sha2_512(sha2::Sha512)
}

impl Default for Hasher {
    #[inline]
    fn default() -> Self {
        Self::Seahash(Default::default())
    }
}

impl Hasher {
    pub fn from_name(name: impl AsRef<str>) -> Option<Self> {
        match name.as_ref() {
            "seahash"  => Some(Self::Seahash(Default::default())),
            "crc32"    => Some(Self::Crc32(Default::default())),
            "crc32c"   => Some(Self::Crc32c(Default::default())),
            "xxh32"    => Some(Self::Xxh32(Default::default())),
            "xxh64"    => Some(Self::Xxh64(Default::default())),
            "xxh3-64"  => Some(Self::Xxh3_64(Default::default())),
            "xxh3-128" => Some(Self::Xxh3_128(Default::default())),
            "md5"      => Some(Self::Md5(md5::Context::new())),
            "sha1"     => Some(Self::Sha1(Default::default())),
            "sha2-224" => Some(Self::Sha2_224(Default::default())),
            "sha2-256" => Some(Self::Sha2_256(Default::default())),
            "sha2-384" => Some(Self::Sha2_384(Default::default())),
            "sha2-512" => Some(Self::Sha2_512(Default::default())),

            _ => None
        }
    }

    pub fn write(&mut self, slice: impl AsRef<[u8]>) -> std::io::Result<()> {
        use std::hash::Hasher;

        match self {
            Self::Seahash(hasher) => hasher.write(slice.as_ref()),
            Self::Crc32(hasher) => hasher.update(slice.as_ref()),
            Self::Crc32c(hasher) => hasher.write(slice.as_ref()),
            Self::Xxh32(hasher) => hasher.update(slice.as_ref()),
            Self::Xxh64(hasher) => hasher.update(slice.as_ref()),
            Self::Xxh3_64(hasher) => hasher.update(slice.as_ref()),
            Self::Xxh3_128(hasher) => hasher.update(slice.as_ref()),
            Self::Md5(hasher) => hasher.write_all(slice.as_ref())?,
            Self::Sha1(hasher) => hasher.write_all(slice.as_ref())?,
            Self::Sha2_224(hasher) => hasher.write_all(slice.as_ref())?,
            Self::Sha2_256(hasher) => hasher.write_all(slice.as_ref())?,
            Self::Sha2_384(hasher) => hasher.write_all(slice.as_ref())?,
            Self::Sha2_512(hasher) => hasher.write_all(slice.as_ref())?
        };

        Ok(())
    }

    pub fn finalize(self) -> Vec<u8> {
        use std::hash::Hasher;
        use sha1::Digest;

        match self {
            Self::Seahash(hasher) => hasher.finish()
                .to_be_bytes()
                .to_vec(),

            Self::Crc32(hasher) => hasher.finalize()
                .to_be_bytes()
                .to_vec(),

            Self::Crc32c(hasher) => (hasher.finish() as u32)
                .to_be_bytes()
                .to_vec(),

            Self::Xxh32(hasher) => hasher.digest()
                .to_be_bytes()
                .to_vec(),

            Self::Xxh64(hasher) => hasher.finish()
                .to_be_bytes()
                .to_vec(),

            Self::Xxh3_64(hasher) => hasher.digest()
                .to_be_bytes()
                .to_vec(),

            Self::Xxh3_128(hasher) => hasher.digest128()
                .to_be_bytes()
                .to_vec(),

            Self::Md5(hasher) => hasher.compute().to_vec(),
            Self::Sha1(hasher) => hasher.finalize().to_vec(),
            Self::Sha2_224(hasher) => hasher.finalize().to_vec(),
            Self::Sha2_256(hasher) => hasher.finalize().to_vec(),
            Self::Sha2_384(hasher) => hasher.finalize().to_vec(),
            Self::Sha2_512(hasher) => hasher.finalize().to_vec()
        }
    }

    pub fn calc(mut self, slice: impl AsRef<[u8]>) -> std::io::Result<Vec<u8>> {
        self.write(slice)?;

        Ok(self.finalize())
    }
}

pub struct HashAPI<'lua> {
    lua: &'lua Lua,

    hash_calc: LuaFunction<'lua>,
    hash_builder: LuaFunction<'lua>,
    hash_write: LuaFunction<'lua>,
    hash_finalize: LuaFunction<'lua>
}

impl<'lua> HashAPI<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, EngineError> {
        let hasher_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            lua,

            hash_calc: lua.create_function(move |_, (value, algorithm): (LuaValue, Option<LuaString>)| {
                let hasher = match algorithm {
                    Some(name) => Hasher::from_name(name.to_string_lossy())
                        .ok_or_else(|| LuaError::external("invalid hash algorithm name"))?,

                    None => Hasher::default()
                };

                Ok(hasher.calc(get_value_bytes(value)?)?)
            })?,

            hash_builder: {
                let hasher_handles = hasher_handles.clone();

                lua.create_function(move |_, algorithm: Option<LuaString>| {
                    let hasher = match algorithm {
                        Some(name) => Hasher::from_name(name.to_string_lossy())
                            .ok_or_else(|| LuaError::external("invalid hash algorithm name"))?,

                        None => Hasher::default()
                    };

                    let mut hashers = hasher_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                    let mut handle = rand::random::<u32>();

                    while hashers.contains_key(&handle) {
                        handle = rand::random::<u32>();
                    }

                    hashers.insert(handle, hasher);

                    Ok(handle)
                })?
            },

            hash_write: {
                let hasher_handles = hasher_handles.clone();

                lua.create_function(move |_, (handle, value): (u32, LuaValue)| {
                    let mut hashers = hasher_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(hasher) = hashers.get_mut(&handle) else {
                        return Err(LuaError::external("invalid hasher handle"));
                    };

                    hasher.write(get_value_bytes(value)?)?;

                    Ok(())
                })?
            },

            hash_finalize: {
                let hasher_handles = hasher_handles.clone();

                lua.create_function(move |_, handle: u32| {
                    let mut hashers = hasher_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(hasher) = hashers.remove(&handle) else {
                        return Err(LuaError::external("invalid hasher handle"));
                    };

                    Ok(hasher.finalize())
                })?
            }
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable<'lua>, EngineError> {
        let env = self.lua.create_table_with_capacity(0, 4)?;

        env.set("calc", self.hash_calc.clone())?;
        env.set("builder", self.hash_builder.clone())?;
        env.set("write", self.hash_write.clone())?;
        env.set("finalize", self.hash_finalize.clone())?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_calc() -> anyhow::Result<()> {
        let lua = Lua::new();
        let api = HashAPI::new(&lua)?;

        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(0.5)?,             &[120, 18, 28, 179, 226, 204, 30, 109]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(-17)?,             &[176, 134, 88, 13, 238, 58, 194, 165]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>("Hello, World!")?, &[46, 194, 87, 41, 102, 208, 6, 253]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(vec![1, 2, 3])?,   &[142, 143, 212, 110, 1, 110, 210, 66]);

        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "seahash"))?,  &[46, 194, 87, 41, 102, 208, 6, 253]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "crc32"))?,    &[236, 74, 195, 208]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "crc32c"))?,   &[77, 85, 16, 104]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "xxh32"))?,    &[64, 7, 222, 80]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "xxh64"))?,    &[196, 154, 172, 248, 8, 15, 228, 127]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "xxh3-64"))?,  &[96, 65, 93, 95, 97, 102, 2, 170]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "xxh3-128"))?, &[83, 29, 242, 132, 68, 71, 221, 80, 119, 219, 3, 132, 44, 215, 83, 149]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "md5"))?,      &[101, 168, 226, 125, 136, 121, 40, 56, 49, 182, 100, 189, 139, 127, 10, 212]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "sha1"))?,     &[10, 10, 159, 42, 103, 114, 148, 37, 87, 171, 83, 85, 215, 106, 244, 66, 248, 246, 94, 1]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "sha2-224"))?, &[114, 162, 61, 250, 65, 27, 166, 253, 224, 29, 191, 171, 243, 176, 10, 112, 156, 147, 235, 242, 115, 220, 41, 226, 216, 178, 97, 255]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "sha2-256"))?, &[223, 253, 96, 33, 187, 43, 213, 176, 175, 103, 98, 144, 128, 158, 195, 165, 49, 145, 221, 129, 199, 247, 10, 75, 40, 104, 138, 54, 33, 130, 152, 111]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "sha2-384"))?, &[84, 133, 204, 155, 51, 101, 180, 48, 93, 251, 78, 131, 55, 224, 165, 152, 165, 116, 248, 36, 43, 241, 114, 137, 224, 221, 108, 32, 163, 205, 68, 160, 137, 222, 22, 171, 74, 179, 8, 246, 62, 68, 177, 23, 14, 181, 245, 21]);
        assert_eq!(api.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "sha2-512"))?, &[55, 77, 121, 74, 149, 205, 207, 216, 179, 89, 147, 24, 95, 239, 155, 163, 104, 241, 96, 216, 218, 244, 50, 208, 139, 169, 241, 237, 30, 90, 190, 108, 198, 146, 145, 224, 250, 47, 224, 0, 106, 82, 87, 14, 241, 140, 25, 222, 244, 230, 23, 195, 60, 229, 46, 240, 166, 229, 251, 227, 24, 203, 3, 135]);

        Ok(())
    }

    #[test]
    fn hash_builder() -> anyhow::Result<()> {
        let lua = Lua::new();
        let api = HashAPI::new(&lua)?;

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
            let hasher = api.hash_builder.call::<_, u32>(name)?;

            api.hash_write.call::<_, ()>((hasher, "Hello"))?;
            api.hash_write.call::<_, ()>((hasher, ", "))?;
            api.hash_write.call::<_, ()>((hasher, "World"))?;
            api.hash_write.call::<_, ()>((hasher, "!"))?;

            assert_eq!(api.hash_finalize.call::<_, Vec<u8>>(hasher)?, hash);
        }

        Ok(())
    }
}
