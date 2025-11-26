use std::str::FromStr;
use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, Mutex, MutexGuard};
use std::io::{BufReader, Cursor, Read, Write};

use mlua::prelude::*;

use super::filesystem_api::IO_READ_CHUNK_LEN;
use super::*;

#[derive(Debug, Clone)]
struct ReadWriteMutex<T: Read + Write>(Arc<Mutex<T>>);

impl<T: Read + Write> ReadWriteMutex<T> {
    #[inline]
    pub fn new(inner: T) -> Self {
        Self(Arc::new(Mutex::new(inner)))
    }

    pub fn inner(&mut self) -> std::io::Result<MutexGuard<'_, T>> {
        self.0.lock()
            .map_err(|err| {
                std::io::Error::other(format!("failed to lock mutex: {err}"))
            })
    }
}

impl<T: Read + Write> Read for ReadWriteMutex<T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner()?.read(buf)
    }

    #[inline]
    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.inner()?.read_vectored(bufs)
    }

    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.inner()?.read_to_end(buf)
    }

    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.inner()?.read_to_string(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.inner()?.read_exact(buf)
    }
}

impl<T: Read + Write> Write for ReadWriteMutex<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner()?.write(buf)
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.inner()?.write_all(buf)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner()?.flush()
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.inner()?.write_vectored(bufs)
    }

    #[inline]
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.inner()?.write_fmt(args)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Level {
    Quick,
    Fast,
    Balanced,
    Good,
    Best,
    #[default]
    Default,
    Custom(i8)
}

impl Level {
    pub const fn zstd_level(&self) -> i32 {
        match self {
            Self::Quick    => 3,
            Self::Fast     => 9,
            Self::Balanced => 13,
            Self::Good     => 17,
            Self::Best     => 22,
            Self::Default  => 10,

            Self::Custom(level) => *level as i32
        }
    }

    pub fn lzma2_options(&self) -> lzma_rust::LZMA2Options {
        lzma_rust::LZMA2Options::with_preset(match self {
            Self::Quick    => 1,
            Self::Fast     => 3,
            Self::Balanced => 5,
            Self::Good     => 7,
            Self::Best     => 9,
            Self::Default  => 4,

            Self::Custom(level) => *level as u32
        })
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quick    => f.write_str("quick"),
            Self::Fast     => f.write_str("fast"),
            Self::Balanced => f.write_str("balanced"),
            Self::Good     => f.write_str("good"),
            Self::Best     => f.write_str("best"),
            Self::Default  => f.write_str("default"),

            Self::Custom(level) => write!(f, "{level}")
        }
    }
}

impl FromStr for Level {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "quick"    => Ok(Self::Quick),
            "fast"     => Ok(Self::Fast),
            "balanced" => Ok(Self::Balanced),
            "good"     => Ok(Self::Good),
            "best"     => Ok(Self::Best),
            "default"  => Ok(Self::Default),

            _ => {
                let Ok(level) = s.parse::<i8>() else {
                    anyhow::bail!("invalid compression level: '{s}'");
                };

                Ok(Self::Custom(level))
            }
        }
    }
}

impl From<Level> for bzip2::Compression {
    fn from(value: Level) -> Self {
        match value {
            Level::Quick    => Self::new(1),
            Level::Fast     => Self::new(3),
            Level::Balanced => Self::new(5),
            Level::Good     => Self::new(7),
            Level::Best     => Self::new(9),
            Level::Default  => Self::new(4),

            Level::Custom(level) => Self::new(level as u32)
        }
    }
}

impl From<Level> for flate2::Compression {
    fn from(value: Level) -> Self {
        match value {
            Level::Quick    => Self::new(1),
            Level::Fast     => Self::new(3),
            Level::Balanced => Self::new(5),
            Level::Good     => Self::new(7),
            Level::Best     => Self::new(9),
            Level::Default  => Self::new(6),

            Level::Custom(level) => Self::new(level as u32)
        }
    }
}

enum Compressor {
    Lz4(lz4_flex::frame::FrameEncoder<Cursor<Vec<u8>>>),
    Bzip(bzip2::write::BzEncoder<Cursor<Vec<u8>>>),
    Deflate(flate2::write::DeflateEncoder<Cursor<Vec<u8>>>),
    Gzip(flate2::write::GzEncoder<Cursor<Vec<u8>>>),
    Zlib(flate2::write::ZlibEncoder<Cursor<Vec<u8>>>),
    Zstd(zstd::Encoder<'static, Cursor<Vec<u8>>>),
    Lzma {
        compressor: lzma_rust::LZMAWriter<ReadWriteMutex<Cursor<Vec<u8>>>>,
        buf: ReadWriteMutex<Cursor<Vec<u8>>>
    },
    Lzma2 {
        compressor: lzma_rust::LZMA2Writer<ReadWriteMutex<Cursor<Vec<u8>>>>,
        buf: ReadWriteMutex<Cursor<Vec<u8>>>
    }
}

impl Compressor {
    pub fn write(&mut self, buf: impl AsRef<[u8]>) -> std::io::Result<()> {
        match self {
            Self::Lz4(compressor)     => compressor.write_all(buf.as_ref())?,
            Self::Bzip(compressor)    => compressor.write_all(buf.as_ref())?,
            Self::Deflate(compressor) => compressor.write_all(buf.as_ref())?,
            Self::Gzip(compressor)    => compressor.write_all(buf.as_ref())?,
            Self::Zlib(compressor)    => compressor.write_all(buf.as_ref())?,
            Self::Zstd(compressor)    => compressor.write_all(buf.as_ref())?,

            Self::Lzma { compressor, .. }  => compressor.write_all(buf.as_ref())?,
            Self::Lzma2 { compressor, .. } => compressor.write_all(buf.as_ref())?
        }

        Ok(())
    }

    pub fn flush(&mut self) -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::new();

        match self {
            Self::Lz4(compressor) => {
                compressor.flush()?;

                compressor.get_mut()
                    .read_to_end(&mut buf)?;
            }

            Self::Bzip(compressor) => {
                compressor.flush()?;

                compressor.get_mut()
                    .read_to_end(&mut buf)?;
            }

            Self::Deflate(compressor) => {
                compressor.flush()?;

                compressor.get_mut()
                    .read_to_end(&mut buf)?;
            }

            Self::Gzip(compressor) => {
                compressor.flush()?;

                compressor.get_mut()
                    .read_to_end(&mut buf)?;
            }

            Self::Zlib(compressor) => {
                compressor.flush()?;

                compressor.get_mut()
                    .read_to_end(&mut buf)?;
            }

            Self::Zstd(compressor) => {
                compressor.flush()?;

                compressor.get_mut()
                    .read_to_end(&mut buf)?;
            }

            Self::Lzma { compressor, buf: compressor_buf } => {
                compressor.flush()?;

                compressor_buf.read_to_end(&mut buf)?;
            }

            Self::Lzma2 { compressor, buf: compressor_buf } => {
                compressor.flush()?;

                compressor_buf.read_to_end(&mut buf)?;
            }
        }

        Ok(buf)
    }
}

impl Default for Compressor {
    fn default() -> Self {
        let buf = Cursor::new(Vec::new());
        let level = Level::default();

        let compressor = zstd::Encoder::new(buf, level.zstd_level())
            .expect("failed to initialize zstd compressor");

        Self::Zstd(compressor)
    }
}

impl FromStr for Compressor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, level) = s.split_once(":")
            .unwrap_or((s, "default"));

        let level = Level::from_str(level)?;

        let buf = Cursor::new(Vec::new());

        match name {
            "lz4" => {
                let compressor = lz4_flex::frame::FrameEncoder::new(buf);

                Ok(Self::Lz4(compressor))
            }

            "bzip" => {
                let compressor = bzip2::write::BzEncoder::new(buf, level.into());

                Ok(Self::Bzip(compressor))
            }

            "deflate" => {
                let compressor = flate2::write::DeflateEncoder::new(buf, level.into());

                Ok(Self::Deflate(compressor))
            }

            "gzip" => {
                let compressor = flate2::write::GzEncoder::new(buf, level.into());

                Ok(Self::Gzip(compressor))
            }

            "zlib" => {
                let compressor = flate2::write::ZlibEncoder::new(buf, level.into());

                Ok(Self::Zlib(compressor))
            }

            "zstd" => {
                let compressor = zstd::Encoder::new(buf, level.zstd_level())?;

                Ok(Self::Zstd(compressor))
            }

            "lzma" => {
                let buf = ReadWriteMutex::new(buf);

                let compressor = lzma_rust::LZMAWriter::new(
                    lzma_rust::CountingWriter::new(buf.clone()),
                    &level.lzma2_options(),
                    true,
                    true,
                    None
                )?;

                Ok(Self::Lzma {
                    compressor,
                    buf
                })
            }

            "lzma2" => {
                let buf = ReadWriteMutex::new(buf);

                let compressor = lzma_rust::LZMA2Writer::new(
                    lzma_rust::CountingWriter::new(buf.clone()),
                    &level.lzma2_options()
                );

                Ok(Self::Lzma2 {
                    compressor,
                    buf
                })
            }

            _ => anyhow::bail!("unknown compression algorithm: {name}")
        }
    }
}

enum Decompressor {
    Lz4(lz4_flex::frame::FrameDecoder<Cursor<Vec<u8>>>),
    Bzip(bzip2::read::MultiBzDecoder<Cursor<Vec<u8>>>),
    Deflate(flate2::read::DeflateDecoder<Cursor<Vec<u8>>>),
    Gzip(flate2::read::MultiGzDecoder<Cursor<Vec<u8>>>),
    Zlib(flate2::read::ZlibDecoder<Cursor<Vec<u8>>>),
    Zstd(zstd::Decoder<'static, BufReader<Cursor<Vec<u8>>>>),
    Lzma {
        decompressor: lzma_rust::LZMAReader<ReadWriteMutex<Cursor<Vec<u8>>>>,
        buf: ReadWriteMutex<Cursor<Vec<u8>>>
    },
    Lzma2 {
        decompressor: lzma_rust::LZMA2Reader<ReadWriteMutex<Cursor<Vec<u8>>>>,
        buf: ReadWriteMutex<Cursor<Vec<u8>>>
    }
}

impl Default for Decompressor {
    fn default() -> Self {
        let buf = Cursor::new(Vec::new());

        let decompressor = zstd::Decoder::new(buf)
            .expect("failed to initialize zstd decompressor");

        Self::Zstd(decompressor)
    }
}

impl FromStr for Decompressor {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, level) = s.split_once(":")
            .unwrap_or((s, "default"));

        let level = Level::from_str(level)?;

        let buf = Cursor::new(Vec::new());

        match name {
            "lz4" => {
                let decompressor = lz4_flex::frame::FrameDecoder::new(buf);

                Ok(Self::Lz4(decompressor))
            }

            "bzip" => {
                let decompressor = bzip2::read::MultiBzDecoder::new(buf);

                Ok(Self::Bzip(decompressor))
            }

            "deflate" => {
                let decompressor = flate2::read::DeflateDecoder::new(buf);

                Ok(Self::Deflate(decompressor))
            }

            "gzip" => {
                let decompressor = flate2::read::MultiGzDecoder::new(buf);

                Ok(Self::Gzip(decompressor))
            }

            "zlib" => {
                let decompressor = flate2::read::ZlibDecoder::new(buf);

                Ok(Self::Zlib(decompressor))
            }

            "zstd" => {
                let decompressor = zstd::Decoder::new(buf)?;

                Ok(Self::Zstd(decompressor))
            }

            "lzma" => {
                let buf = ReadWriteMutex::new(buf);

                let decompressor = lzma_rust::LZMAReader::new_mem_limit(buf.clone(), 1024 * 1024, None)?;

                Ok(Self::Lzma {
                    decompressor,
                    buf
                })
            }

            "lzma2" => {
                let buf = ReadWriteMutex::new(buf);

                let options = level.lzma2_options();

                let decompressor = lzma_rust::LZMA2Reader::new(buf.clone(), options.dict_size, None);

                Ok(Self::Lzma2 {
                    decompressor,
                    buf
                })
            }

            _ => anyhow::bail!("unknown compression algorithm: {name}")
        }
    }
}

pub struct CompressionAPI {
    lua: Lua,

    compression_compress: LuaFunction,
    compression_decompress: LuaFunction,
    compression_compressor: LuaFunction,
    compression_decompressor: LuaFunction,
    compression_write: LuaFunction,
    compression_flush: LuaFunction,
    compression_close: LuaFunction
}

impl CompressionAPI {
    pub fn new(lua: Lua) -> Result<Self, PackagesEngineError> {
        let hasher_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            compression_compress: lua.create_function(move |_, (value, algorithm): (LuaValue, Option<LuaString>)| {
                let compressor = match algorithm {
                    Some(name) => Compressor::from_str(name.to_string_lossy())
                        .ok_or_else(|| LuaError::external("invalid hash algorithm name"))?,

                    None => Compressor::default()
                };

                Ok(hasher.calc(get_value_bytes(value)?)?)
            })?,

            hash_file: {
                Box::new(move |lua: &Lua, context: &Context| {
                    let context = context.to_owned();

                    lua.create_function(move |_, (path, algorithm): (LuaString, Option<LuaString>)| {
                        let mut path = resolve_path(path.to_string_lossy())?;

                        if path.is_relative() {
                            path = context.module_folder.join(path);
                        }

                        if !context.is_accessible(&path) {
                            return Err(LuaError::external("path is inaccessible"));
                        }

                        let mut file = File::open(path)?;

                        let mut hasher = match algorithm {
                            Some(name) => Hasher::from_name(name.to_string_lossy())
                                .ok_or_else(|| LuaError::external("invalid hash algorithm name"))?,

                            None => Hasher::default()
                        };

                        let mut buf = [0; IO_READ_CHUNK_LEN];

                        loop {
                            let n = file.read(&mut buf)?;

                            if n == 0 {
                                break;
                            }

                            hasher.write(&buf[..n])?;
                        }

                        Ok(hasher.finalize())
                    })
                })
            },

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

                    let mut handle = rand::random::<i32>();

                    while hashers.contains_key(&handle) {
                        handle = rand::random::<i32>();
                    }

                    hashers.insert(handle, hasher);

                    Ok(handle)
                })?
            },

            hash_write: {
                let hasher_handles = hasher_handles.clone();

                lua.create_function(move |_, (handle, value): (i32, LuaValue)| {
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

                lua.create_function(move |_, handle: i32| {
                    let mut hashers = hasher_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(hasher) = hashers.remove(&handle) else {
                        return Err(LuaError::external("invalid hasher handle"));
                    };

                    Ok(hasher.finalize())
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

        env.raw_set("calc", self.hash_calc.clone())?;
        env.raw_set("file", (self.hash_file)(&self.lua, context)?)?;
        env.raw_set("builder", self.hash_builder.clone())?;
        env.raw_set("write", self.hash_write.clone())?;
        env.raw_set("finalize", self.hash_finalize.clone())?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_calc() -> anyhow::Result<()> {
        let api = HashAPI::new(Lua::new())?;

        assert_eq!(api.hash_calc.call::<Vec<u8>>(0.5)?,             &[120, 18, 28, 179, 226, 204, 30, 109]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(-17)?,             &[176, 134, 88, 13, 238, 58, 194, 165]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>("Hello, World!")?, &[46, 194, 87, 41, 102, 208, 6, 253]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(vec![1, 2, 3])?,   &[142, 143, 212, 110, 1, 110, 210, 66]);

        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "seahash"))?,  &[46, 194, 87, 41, 102, 208, 6, 253]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "crc32"))?,    &[236, 74, 195, 208]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "crc32c"))?,   &[77, 85, 16, 104]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "xxh32"))?,    &[64, 7, 222, 80]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "xxh64"))?,    &[196, 154, 172, 248, 8, 15, 228, 127]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "xxh3-64"))?,  &[96, 65, 93, 95, 97, 102, 2, 170]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "xxh3-128"))?, &[83, 29, 242, 132, 68, 71, 221, 80, 119, 219, 3, 132, 44, 215, 83, 149]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "md5"))?,      &[101, 168, 226, 125, 136, 121, 40, 56, 49, 182, 100, 189, 139, 127, 10, 212]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "sha1"))?,     &[10, 10, 159, 42, 103, 114, 148, 37, 87, 171, 83, 85, 215, 106, 244, 66, 248, 246, 94, 1]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "sha2-224"))?, &[114, 162, 61, 250, 65, 27, 166, 253, 224, 29, 191, 171, 243, 176, 10, 112, 156, 147, 235, 242, 115, 220, 41, 226, 216, 178, 97, 255]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "sha2-256"))?, &[223, 253, 96, 33, 187, 43, 213, 176, 175, 103, 98, 144, 128, 158, 195, 165, 49, 145, 221, 129, 199, 247, 10, 75, 40, 104, 138, 54, 33, 130, 152, 111]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "sha2-384"))?, &[84, 133, 204, 155, 51, 101, 180, 48, 93, 251, 78, 131, 55, 224, 165, 152, 165, 116, 248, 36, 43, 241, 114, 137, 224, 221, 108, 32, 163, 205, 68, 160, 137, 222, 22, 171, 74, 179, 8, 246, 62, 68, 177, 23, 14, 181, 245, 21]);
        assert_eq!(api.hash_calc.call::<Vec<u8>>(("Hello, World!", "sha2-512"))?, &[55, 77, 121, 74, 149, 205, 207, 216, 179, 89, 147, 24, 95, 239, 155, 163, 104, 241, 96, 216, 218, 244, 50, 208, 139, 169, 241, 237, 30, 90, 190, 108, 198, 146, 145, 224, 250, 47, 224, 0, 106, 82, 87, 14, 241, 140, 25, 222, 244, 230, 23, 195, 60, 229, 46, 240, 166, 229, 251, 227, 24, 203, 3, 135]);

        Ok(())
    }

    #[test]
    fn hash_builder() -> anyhow::Result<()> {
        let api = HashAPI::new(Lua::new())?;

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
            let hasher = api.hash_builder.call::<i32>(name)?;

            api.hash_write.call::<()>((hasher, "Hello"))?;
            api.hash_write.call::<()>((hasher, ", "))?;
            api.hash_write.call::<()>((hasher, "World"))?;
            api.hash_write.call::<()>((hasher, "!"))?;

            assert_eq!(api.hash_finalize.call::<Vec<u8>>(hasher)?, hash);
        }

        Ok(())
    }
}
