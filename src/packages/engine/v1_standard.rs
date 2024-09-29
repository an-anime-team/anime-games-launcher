use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{UNIX_EPOCH, Duration};
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Write, Seek, SeekFrom};
use std::process::{Command, Stdio};

use mlua::prelude::*;
use tokio::runtime::Runtime;
use encoding_rs::Encoding;
use bufreaderwriter::rand::BufReaderWriterRand;
use reqwest::{Client, Method};

use crate::core::prelude::*;
use crate::packages::prelude::*;
use crate::config;

use super::EngineError;

const NET_READ_CHUNK_LEN: usize = 8192;

// TODO: should get its own config field.
const NET_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

const MUTEX_LOCK_TIMEOUT: Duration = Duration::from_millis(100);

const PROCESS_READ_CHUNK_LEN: usize = 1024;

lazy_static::lazy_static! {
    static ref NET_RUNTIME: Runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("v1_net_api")
        .enable_all()
        .build()
        .expect("Failed to create v1 network API tasks runtime");
}

fn resolve_path(path: impl AsRef<str>) -> std::io::Result<PathBuf> {
    let mut path = PathBuf::from(path.as_ref());

    while path.is_symlink() {
        path = path.read_link()?;
    }

    Ok(path)
}

fn normalize_path_parts(parts: &[impl AsRef<str>]) -> Option<Vec<String>> {
    let mut normal_parts = Vec::with_capacity(parts.len());

    let mut i = 0;
    let n = parts.len();

    while i < n {
        let part = parts[i].as_ref();

        if part == "." {
            i += 1;
        }

        else if part == ".." {
            normal_parts.pop()?;

            i += 1;
        }

        else {
            if !["", "/", "\\"].contains(&part) {
                normal_parts.push(part.to_string());
            }

            i += 1;
        }
    }

    if normal_parts.is_empty() {
        None
    } else {
        Some(normal_parts)
    }
}

fn split_path(path: impl AsRef<str>) -> Option<Vec<String>> {
    let path = path.as_ref()
        .replace('\\', "/");

    let raw_parts = path
        .split('/')
        .collect::<Vec<_>>();

    normalize_path_parts(&raw_parts)
}

fn create_request(client: &Arc<Client>, url: impl AsRef<str>, options: Option<LuaTable>) -> Result<reqwest::RequestBuilder, LuaError> {
    let mut method = String::from("get");
    
    // Change the request method if provided.
    if let Some(options) = &options {
        method = options.get::<_, String>("method")
            .unwrap_or(String::from("get"));
    }

    let method = match method.to_ascii_lowercase().as_str() {
        "get"     => Method::GET,
        "port"    => Method::POST,
        "head"    => Method::HEAD,
        "put"     => Method::PUT,
        "patch"   => Method::PATCH,
        "delete"  => Method::DELETE,
        "connect" => Method::CONNECT,

        _ => return Err(LuaError::external("invalid request method"))
    };

    let mut request = client.request(method, url.as_ref());

    // Set request header and body if provided.
    if let Some(options) = &options {
        if let Ok(headers) = options.get::<_, LuaTable>("headers") {
            for pair in headers.pairs::<LuaString, LuaString>() {
                let (key, value) = pair?;

                request = request.header(
                    key.to_string_lossy().to_string(),
                    value.to_string_lossy().to_string()
                );
            }
        }

        if let Ok(body) = options.get::<_, LuaValue>("body") {
            request = match body {
                LuaValue::String(str) => request.body(str.as_bytes().to_vec()),

                LuaValue::Table(table) => {
                    let mut body = Vec::with_capacity(table.raw_len());

                    for byte in table.sequence_values::<u8>() {
                        body.push(byte?);
                    }

                    request.body(body)
                }

                _ => return Err(LuaError::external("invalid body value"))
            };
        }
    }

    Ok(request)
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

#[allow(clippy::large_enum_variant)]
enum StringEncoding {
    Base16,
    Base32(base32::Alphabet),
    Base64(base64::engine::GeneralPurpose),
    Json
}

impl StringEncoding {
    pub fn from_name(name: impl AsRef<[u8]>) -> Option<Self> {
        match name.as_ref() {
            b"base16" | b"hex" => Some(Self::Base16),

            // Base32
            b"base32" | b"base32/pad" => {
                Some(Self::Base32(base32::Alphabet::Rfc4648Lower { padding: true }))
            }

            b"base32/nopad" => {
                Some(Self::Base32(base32::Alphabet::Rfc4648Lower { padding: false }))
            }

            b"base32/hex-pad"   => {
                Some(Self::Base32(base32::Alphabet::Rfc4648HexLower { padding: true }))
            }

            b"base32/hex-nopad" => {
                Some(Self::Base32(base32::Alphabet::Rfc4648HexLower { padding: false }))
            }

            // Base64
            b"base64" | b"base64/pad" => {
                let encoding = base64::engine::GeneralPurpose::new(
                    &base64::alphabet::STANDARD,
                    base64::engine::GeneralPurposeConfig::new()
                        .with_encode_padding(true)
                );

                Some(Self::Base64(encoding))
            }

            b"base64/nopad" => {
                let encoding = base64::engine::GeneralPurpose::new(
                    &base64::alphabet::STANDARD,
                    base64::engine::GeneralPurposeConfig::new()
                        .with_encode_padding(false)
                );

                Some(Self::Base64(encoding))
            }

            b"base64/urlsafe-pad" => {
                let encoding = base64::engine::GeneralPurpose::new(
                    &base64::alphabet::URL_SAFE,
                    base64::engine::GeneralPurposeConfig::new()
                        .with_encode_padding(true)
                );

                Some(Self::Base64(encoding))
            }

            b"base64/urlsafe-nopad" => {
                let encoding = base64::engine::GeneralPurpose::new(
                    &base64::alphabet::URL_SAFE,
                    base64::engine::GeneralPurposeConfig::new()
                        .with_encode_padding(false)
                );

                Some(Self::Base64(encoding))
            }

            b"json" => Some(Self::Json),

            _ => None
        }
    }

    pub fn encode<'lua>(&self, lua: &'lua Lua, value: LuaValue) -> Result<LuaString<'lua>, LuaError> {
        use base64::Engine;

        match self {
            Self::Base16 => {
                let value = get_value_bytes(value)?;

                lua.create_string(hex::encode(value))
            }

            Self::Base32(alphabet) => {
                let value = get_value_bytes(value)?;

                lua.create_string(base32::encode(*alphabet, &value))
            }

            Self::Base64(engine) => {
                let value = get_value_bytes(value)?;

                lua.create_string(engine.encode(value))
            }

            Self::Json => {
                let value = serde_json::to_vec(&value)
                    .map_err(LuaError::external)?;

                lua.create_string(value)
            }
        }
    }

    pub fn decode<'lua>(&self, lua: &'lua Lua, string: LuaString) -> Result<LuaValue<'lua>, LuaError> {
        use base64::Engine;

        match self {
            Self::Base16 => {
                let value = hex::decode(string.as_bytes())
                    .map_err(LuaError::external)?;

                slice_to_table(lua, value)
                    .map(LuaValue::Table)
            }

            Self::Base32(alphabet) => {
                let string = string.to_string_lossy()
                    .to_string();

                let value = base32::decode(*alphabet, &string)
                    .ok_or_else(|| LuaError::external("invalid base32 string"))?;

                slice_to_table(lua, value)
                    .map(LuaValue::Table)
            }

            Self::Base64(engine) => {
                let value = engine.decode(string.as_bytes())
                    .map_err(LuaError::external)?;

                slice_to_table(lua, value)
                    .map(LuaValue::Table)
            }

            Self::Json => {
                let value = serde_json::from_slice::<serde_json::Value>(string.as_bytes())
                    .map_err(LuaError::external)?;

                lua.to_value(&value)
            }
        }
    }
}

// Workaround for lifetimes fuckery.
#[derive(Debug, Clone)]
enum ChannelMessage {
    Table(Vec<(Self, Self)>),
    String(String),
    Double(f64),
    Integer(i32),
    Boolean(bool),
    Nil
}

impl ChannelMessage {
    pub fn from_lua(value: LuaValue) -> Result<Self, LuaError> {
        match value {
            LuaValue::String(value) => Ok(Self::String(value.to_string_lossy().to_string())),
            LuaValue::Number(value) => Ok(Self::Double(value)),
            LuaValue::Integer(value) => Ok(Self::Integer(value)),
            LuaValue::Boolean(value) => Ok(Self::Boolean(value)),
            LuaValue::Nil => Ok(Self::Nil),

            LuaValue::Table(table) => {
                let mut result = Vec::with_capacity(table.raw_len());

                for pair in table.pairs::<LuaValue, LuaValue>() {
                    let (key, value) = pair?;

                    result.push((
                        Self::from_lua(key)?,
                        Self::from_lua(value)?
                    ));
                }

                Ok(Self::Table(result))
            }

            _ => Err(LuaError::external("can't coerce given value type"))
        }
    }

    pub fn to_lua<'lua>(&self, lua: &'lua Lua) -> Result<LuaValue<'lua>, LuaError> {
        match self {
            Self::String(value) => lua.create_string(value)
                .map(LuaValue::String),

            Self::Double(value) => Ok(LuaValue::Number(*value)),
            Self::Integer(value) => Ok(LuaValue::Integer(*value)),
            Self::Boolean(value) => Ok(LuaValue::Boolean(*value)),
            Self::Nil => Ok(LuaNil),

            Self::Table(table) => {
                let result = lua.create_table_with_capacity(0, table.len())?;

                for (key, value) in table {
                    result.set(
                        key.to_lua(lua)?,
                        value.to_lua(lua)?
                    )?;
                }

                Ok(LuaValue::Table(result))
            }
        }
    }
}

enum Archive {
    Tar(TarArchive),
    Zip(ZipArchive),
    Sevenz(SevenzArchive)
}

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

pub struct Standard<'lua> {
    lua: &'lua Lua,

    clone: LuaFunction<'lua>,

    str_to_bytes: LuaFunction<'lua>,
    str_from_bytes: LuaFunction<'lua>,
    str_encode: LuaFunction<'lua>,
    str_decode: LuaFunction<'lua>,

    path_temp_dir: LuaFunction<'lua>,
    path_module_dir: LuaFunction<'lua>,
    path_persist_dir: LuaFunction<'lua>,
    path_normalize: LuaFunction<'lua>,
    path_join: LuaFunction<'lua>,
    path_parts: LuaFunction<'lua>,
    path_parent: LuaFunction<'lua>,
    path_file_name: LuaFunction<'lua>,
    path_exists: LuaFunction<'lua>,
    path_accessible: LuaFunction<'lua>,

    fs_exists: LuaFunction<'lua>,
    fs_metadata: LuaFunction<'lua>,
    fs_copy: LuaFunction<'lua>,
    fs_move: LuaFunction<'lua>,
    fs_remove: LuaFunction<'lua>,
    fs_open: LuaFunction<'lua>,
    fs_seek: LuaFunction<'lua>,
    fs_read: LuaFunction<'lua>,
    fs_write: LuaFunction<'lua>,
    fs_flush: LuaFunction<'lua>,
    fs_close: LuaFunction<'lua>,

    fs_read_file: LuaFunction<'lua>,
    fs_write_file: LuaFunction<'lua>,
    fs_remove_file: LuaFunction<'lua>,
    fs_create_dir: LuaFunction<'lua>,
    fs_read_dir: LuaFunction<'lua>,
    fs_remove_dir: LuaFunction<'lua>,

    net_fetch: LuaFunction<'lua>,
    net_open: LuaFunction<'lua>,
    net_read: LuaFunction<'lua>,
    net_close: LuaFunction<'lua>,

    sync_channel_open: LuaFunction<'lua>,
    sync_channel_send: LuaFunction<'lua>,
    sync_channel_recv: LuaFunction<'lua>,
    sync_channel_close: LuaFunction<'lua>,

    sync_mutex_open: LuaFunction<'lua>,
    sync_mutex_lock: LuaFunction<'lua>,
    sync_mutex_unlock: LuaFunction<'lua>,
    sync_mutex_close: LuaFunction<'lua>,

    archive_open: LuaFunction<'lua>,
    archive_entries: LuaFunction<'lua>,
    archive_extract: LuaFunction<'lua>,
    archive_close: LuaFunction<'lua>,

    hash_calc: LuaFunction<'lua>,
    hash_builder: LuaFunction<'lua>,
    hash_write: LuaFunction<'lua>,
    hash_finalize: LuaFunction<'lua>,

    // Extended privileges

    process_exec: LuaFunction<'lua>,
    process_open: LuaFunction<'lua>,
    process_stdin: LuaFunction<'lua>,
    process_stdout: LuaFunction<'lua>,
    process_stderr: LuaFunction<'lua>,
    process_kill: LuaFunction<'lua>,
    process_wait: LuaFunction<'lua>,
    process_finished: LuaFunction<'lua>
}

impl<'lua> Standard<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, EngineError> {
        let net_client = Arc::new(Client::builder()
            .connect_timeout(NET_REQUEST_TIMEOUT)
            .build()?);

        let file_handles = Arc::new(Mutex::new(HashMap::new()));
        let net_handles = Arc::new(Mutex::new(HashMap::new()));
        let archive_handles = Arc::new(Mutex::new(HashMap::new()));
        let hasher_handles = Arc::new(Mutex::new(HashMap::new()));
        let process_handles = Arc::new(Mutex::new(HashMap::new()));

        let sync_channels_consumers = Arc::new(Mutex::new(HashMap::new())); // key => handle
        let sync_channels_data = Arc::new(Mutex::new(HashMap::new())); // handle => (key, data)

        let sync_mutex_consumers = Arc::new(Mutex::new(HashMap::<u32, Hash>::new())); // handle => key
        let sync_mutex_locks = Arc::new(Mutex::new(HashMap::<Hash, Option<u32>>::new())); // key => curr_lock_handle

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

            str_to_bytes: lua.create_function(|_, (value, charset): (LuaValue, Option<LuaString>)| {
                let value = get_value_bytes(value)?;

                let Some(charset) = charset else {
                    return Ok(value);
                };

                let Some(charset) = Encoding::for_label(charset.as_bytes()) else {
                    return Err(LuaError::external("invalid charset"));
                };

                let value = String::from_utf8(value)
                    .map_err(|err| LuaError::external(format!("utf-8 string expected: {err}")))?;

                Ok(charset.encode(&value).0.to_vec())
            })?,

            str_from_bytes: lua.create_function(|lua, (value, charset): (Vec<u8>, Option<LuaString>)| {
                let Some(charset) = charset else {
                    return lua.create_string(value);
                };

                let Some(charset) = Encoding::for_label(charset.as_bytes()) else {
                    return Err(LuaError::external("invalid charset"));
                };

                let value = charset.decode(&value).0;

                lua.create_string(value.as_bytes())
            })?,

            str_encode: lua.create_function(|lua, (value, encoding): (LuaValue, LuaString)| {
                let Some(encoding) = StringEncoding::from_name(encoding.as_bytes()) else {
                    return Err(LuaError::external("invalid encoding"));
                };

                encoding.encode(lua, value)
            })?,

            str_decode: lua.create_function(|lua, (value, encoding): (LuaString, LuaString)| {
                let Some(encoding) = StringEncoding::from_name(encoding.as_bytes()) else {
                    return Err(LuaError::external("invalid encoding"));
                };

                encoding.decode(lua, value)
            })?,

            path_temp_dir: lua.create_function(|_, ()| {
                let path = std::env::temp_dir()
                    .to_string_lossy()
                    .to_string();

                Ok(path)
            })?,

            path_module_dir: lua.create_function(|_, ()| {
                let path = config::get().packages.modules_store_path
                    .join("TODO")
                    .to_string_lossy()
                    .to_string();

                Ok(path)
            })?,

            path_persist_dir: lua.create_function(|_, key: LuaString| {
                let path = config::get().packages.persist_store_path
                    .join(Hash::for_slice(key.as_bytes()).to_base32())
                    .to_string_lossy()
                    .to_string();

                Ok(path)
            })?,

            path_normalize: lua.create_function(|lua, path: LuaString| {
                let path = path.to_string_lossy()
                    .to_string();

                if path.is_empty() {
                    return Ok(LuaNil);
                }

                let (path, is_absolute) = match path.strip_prefix("/") {
                    Some(path) => (path, true),
                    None => (path.as_str(), false)
                };

                match split_path(path) {
                    Some(parts) => {
                        let mut path = parts.join("/");

                        if is_absolute {
                            path = format!("/{path}");
                        }

                        lua.create_string(path)
                            .map(LuaValue::String)
                    }

                    None if is_absolute => lua.create_string("/")
                        .map(LuaValue::String),

                    None => Ok(LuaNil)
                }
            })?,

            path_join: lua.create_function(|lua, parts: Vec<LuaString>| {
                if parts.is_empty() {
                    return Ok(LuaNil);
                }

                let parts = parts.iter()
                    .filter(|part| !part.as_bytes().is_empty())
                    .map(LuaString::to_string_lossy)
                    .collect::<Vec<_>>();

                let (parts, is_absolute) = match parts.first() {
                    None => return Ok(LuaNil),

                    Some(v) if v == "/" || v == "\\" => (&parts[1..], true),
                    Some(_) => (parts.as_slice(), false)
                };

                let Some(parts) = normalize_path_parts(parts) else {
                    if is_absolute {
                        return lua.create_string("/")
                            .map(LuaValue::String);
                    } else {
                        return Ok(LuaNil);
                    }
                };

                let mut path = parts.join("/");

                if is_absolute {
                    path = format!("/{path}");
                }

                lua.create_string(path)
                    .map(LuaValue::String)
            })?,

            path_parts: lua.create_function(|lua, path: LuaString| {
                let path = path.to_string_lossy()
                    .to_string();

                if path.is_empty() {
                    return Ok(LuaNil);
                }

                let path = path.strip_prefix("/")
                    .unwrap_or(&path);

                let Some(parts) = split_path(path) else {
                    return Ok(LuaNil);
                };

                let result = lua.create_table_with_capacity(parts.len(), 0)?;

                for part in parts {
                    result.push(part)?;
                }

                Ok(LuaValue::Table(result))
            })?,

            path_parent: lua.create_function(|lua, path: LuaString| {
                let path = path.to_string_lossy()
                    .to_string();

                if path.is_empty() {
                    return Ok(LuaNil);
                }

                let (path, is_absolute) = match path.strip_prefix("/") {
                    Some(path) => (path, true),
                    None => (path.as_str(), false)
                };

                let Some(parts) = split_path(path) else {
                    return Ok(LuaNil);
                };

                if parts.len() > 1 {
                    let mut path = parts[..parts.len() - 1].join("/");

                    if is_absolute {
                        path = format!("/{path}");
                    }

                    lua.create_string(path)
                        .map(LuaValue::String)
                }

                else {
                    Ok(LuaNil)
                }
            })?,

            path_file_name: lua.create_function(|lua, path: LuaString| {
                let path = path.to_string_lossy()
                    .to_string();

                if path.is_empty() {
                    return Ok(LuaNil);
                }

                if path == "/" {
                    return lua.create_string("/")
                        .map(LuaValue::String);
                }

                let path = path.strip_prefix("/")
                    .unwrap_or(&path);

                let Some(mut parts) = split_path(path) else {
                    return Ok(LuaNil);
                };

                let Some(file_name) = parts.pop() else {
                    return Ok(LuaNil);
                };

                lua.create_string(file_name)
                    .map(LuaValue::String)
            })?,

            path_exists: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                Ok(path.exists())
            })?,

            // TODO
            path_accessible: lua.create_function(|_, _path: LuaString| {
                Ok(true)
            })?,

            fs_exists: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                Ok(path.exists())
            })?,

            fs_metadata: lua.create_function(|lua, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                let metadata = path.metadata()?;

                let result = lua.create_table()?;

                result.set("created_at", {
                    metadata.created()?
                        .duration_since(UNIX_EPOCH)
                        .as_ref()
                        .map(Duration::as_secs)
                        .unwrap_or_default()
                })?;

                result.set("modified_at", {
                    metadata.modified()?
                        .duration_since(UNIX_EPOCH)
                        .as_ref()
                        .map(Duration::as_secs)
                        .unwrap_or_default() as u32
                })?;

                result.set("length", metadata.len() as u32)?;
                result.set("is_accessible", true)?; // TODO

                result.set("type", {
                    if metadata.is_symlink() {
                        "symlink"
                    } else if metadata.is_dir() {
                        "folder"
                    } else {
                        "file"
                    }
                })?;

                Ok(result)
            })?,

            fs_copy: lua.create_function(|_, (source, target): (LuaString, LuaString)| {
                let source = resolve_path(source.to_string_lossy())?;
                let target = resolve_path(target.to_string_lossy())?;

                // Throw an error if source path doesn't exists.
                if !source.exists() {
                    return Err(LuaError::external("source path doesn't exists"));
                }

                // Throw an error if target path already exists.
                if target.exists() {
                    return Err(LuaError::external("target path already exists"));
                }

                fn try_copy(source: &Path, target: &Path) -> std::io::Result<()> {
                    if source.is_file() {
                        std::fs::copy(source, target)?;
                    }

                    else if source.is_dir() {
                        std::fs::create_dir_all(target)?;

                        for entry in source.read_dir()? {
                            let entry = entry?;

                            try_copy(&entry.path(), &target.join(entry.file_name()))?;
                        }
                    }

                    else if source.is_symlink() {
                        if let Some(source_filename) = source.file_name() {
                            std::os::unix::fs::symlink(
                                source.read_link()?,
                                target.join(source_filename)
                            )?;
                        }
                    }

                    Ok(())
                }

                try_copy(&source, &target)?;

                Ok(())
            })?,

            fs_move: lua.create_function(|_, (source, target): (LuaString, LuaString)| {
                let source = resolve_path(source.to_string_lossy())?;
                let target = resolve_path(target.to_string_lossy())?;

                // Throw an error if source path doesn't exists.
                if !source.exists() {
                    return Err(LuaError::external("source path doesn't exists"));
                }

                // Throw an error if target path already exists.
                if target.exists() {
                    return Err(LuaError::external("target path already exists"));
                }

                fn try_move(source: &Path, target: &Path) -> std::io::Result<()> {
                    if source.is_file() {
                        // Try to rename the file (mv) or copy
                        // it and then remove the source if mv
                        // has failed (different mounts).
                        if std::fs::rename(source, target).is_err() {
                            std::fs::copy(source, target)?;
                            std::fs::remove_file(source)?;
                        }
                    }

                    else if source.is_dir() {
                        // Try to rename the folder (mv) or create
                        // a target folder and move all the files there.
                        if std::fs::rename(source, target).is_err() {
                            std::fs::create_dir_all(target)?;

                            for entry in source.read_dir()? {
                                let entry = entry?;

                                try_move(&entry.path(), &target.join(entry.file_name()))?;
                            }

                            std::fs::remove_dir_all(source)?;
                        }
                    }

                    else if source.is_symlink() {
                        if let Some(source_filename) = source.file_name() {
                            std::os::unix::fs::symlink(
                                source.read_link()?,
                                target.join(source_filename)
                            )?;
                        }

                        std::fs::remove_file(source)?;
                    }

                    Ok(())
                }

                try_move(&source, &target)?;

                Ok(())
            })?,

            fs_remove: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                // Symlinks are resolved so we don't need to check for them.
                if path.is_file() {
                    std::fs::remove_file(path)?;
                }

                else if path.is_dir() {
                    std::fs::remove_dir_all(path)?;
                }

                Ok(())
            })?,

            fs_open: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (path, options): (LuaString, Option<LuaTable>)| {
                    let path = resolve_path(path.to_string_lossy())?;

                    let mut read = true;
                    let mut write = false;
                    let mut create = false;
                    let mut overwrite = false;
                    let mut append = false;

                    if let Some(options) = options {
                        read      = options.get::<_, bool>("read").unwrap_or(true);
                        write     = options.get::<_, bool>("write").unwrap_or_default();
                        create    = options.get::<_, bool>("create").unwrap_or_default();
                        overwrite = options.get::<_, bool>("overwrite").unwrap_or_default();
                        append    = options.get::<_, bool>("append").unwrap_or_default();
                    }

                    let file = File::options()
                        .read(read)
                        .write(write)
                        .create(create)
                        .truncate(overwrite)
                        .append(append)
                        .open(path)?;

                    let mut handles = file_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                    let mut handle = rand::random::<u32>();

                    while handles.contains_key(&handle) {
                        handle = rand::random::<u32>();
                    }

                    handles.insert(handle, BufReaderWriterRand::new_reader(file));

                    Ok(handle)
                })?
            },

            fs_seek: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, position): (u32, i32)| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(file) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid file handle"));
                    };

                    // Seek the file.
                    if position >= 0 {
                        file.seek(SeekFrom::Start(position as u64))?;
                    }

                    else {
                        file.seek(SeekFrom::End(position as i64))?;
                    }

                    Ok(())
                })?
            },

            fs_read: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, position, length): (u32, Option<i32>, Option<u32>)| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(file) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid file handle"));
                    };

                    // Seek the file if position is given.
                    if let Some(position) = position {
                        if position >= 0 {
                            file.seek(SeekFrom::Start(position as u64))?;
                        }

                        else {
                            file.seek(SeekFrom::End(position as i64))?;
                        }
                    }

                    // Read exact amount of bytes.
                    if let Some(length) = length {
                        let mut buf = vec![0; length as usize];

                        file.read_exact(&mut buf)?;

                        Ok(buf)
                    }

                    // Or just read a chunk of data.
                    else {
                        let mut buf = [0; NET_READ_CHUNK_LEN];

                        let len = file.read(&mut buf)?;

                        Ok(buf[..len].to_vec())
                    }
                })?
            },

            fs_write: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, (handle, content, position): (u32, Vec<u8>, Option<i32>)| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(file) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid file handle"));
                    };

                    // Seek the file if position is given.
                    if let Some(position) = position {
                        if position >= 0 {
                            file.seek(SeekFrom::Start(position as u64))?;
                        }

                        else {
                            file.seek(SeekFrom::End(position as i64))?;
                        }
                    }

                    // Write the content to the file.
                    file.write_all(&content)?;

                    Ok(())
                })?
            },

            fs_flush: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, handle: u32| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    // Flush the file if the handle is valid.
                    if let Some(file) = handles.get_mut(&handle) {
                        file.flush()?;
                    }

                    Ok(())
                })?
            },

            fs_close: {
                let file_handles = file_handles.clone();

                lua.create_function(move |_, handle: u32| {
                    let mut handles = file_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    // Flush the file if the handle is valid.
                    if let Some(file) = handles.get_mut(&handle) {
                        file.flush()?;
                    }

                    // Remove the file handle.
                    handles.remove(&handle);

                    Ok(())
                })?
            },

            fs_read_file: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                Ok(std::fs::read(path)?)
            })?,

            fs_write_file: lua.create_function(|_, (path, content): (LuaString, Vec<u8>)| {
                let path = resolve_path(path.to_string_lossy())?;

                std::fs::write(path, &content)?;

                Ok(())
            })?,

            fs_remove_file: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                std::fs::remove_file(path)?;

                Ok(())
            })?,

            fs_create_dir: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                std::fs::create_dir_all(path)?;

                Ok(())
            })?,

            fs_read_dir: lua.create_function(|lua, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                let entries = lua.create_table()?;

                for entry in path.read_dir()? {
                    let entry = entry?;
                    let entry_table = lua.create_table()?;

                    entry_table.set("name", entry.file_name().to_string_lossy().to_string())?;
                    entry_table.set("path", entry.path().to_string_lossy().to_string())?;

                    entry_table.set("type", {
                        if entry.path().is_symlink() {
                            "symlink"
                        } else if entry.path().is_dir() {
                            "folder"
                        } else {
                            "file"
                        }
                    })?;

                    entries.push(entry_table)?;
                }

                Ok(entries)
            })?,

            fs_remove_dir: lua.create_function(|_, path: LuaString| {
                let path = resolve_path(path.to_string_lossy())?;

                std::fs::remove_dir_all(path)?;

                Ok(())
            })?,

            net_fetch: {
                let net_client = net_client.clone();

                lua.create_function(move |lua, (url, options): (LuaString, Option<LuaTable>)| {
                    let url = url.to_string_lossy().to_string();
                    let request = create_request(&net_client, url, options)?;

                    // Perform the request.
                    let response = NET_RUNTIME.block_on(async move {
                        let result = lua.create_table()?;
                        let headers = lua.create_table()?;

                        let response = request.send().await
                            .map_err(|err| LuaError::external(format!("failed to perform request: {err}")))?;

                        result.set("status", response.status().as_u16())?;
                        result.set("is_ok", response.status().is_success())?;
                        result.set("headers", headers.clone())?;

                        for (key, value) in response.headers() {
                            headers.set(key.to_string(), lua.create_string(value.as_bytes())?)?;
                        }

                        let body = response.bytes().await
                            .map_err(|err| LuaError::external(format!("failed to fetch body: {err}")))?;

                        result.set("body", body.to_vec())?;

                        Ok::<_, LuaError>(result)
                    })?;

                    Ok(response)
                })?
            },

            net_open: {
                let net_client = net_client.clone();
                let net_handles = net_handles.clone();

                lua.create_function(move |lua, (url, options): (LuaString, Option<LuaTable>)| {
                    let url = url.to_string_lossy().to_string();
                    let request = create_request(&net_client, url, options)?;

                    let (response, header) = NET_RUNTIME.block_on(async move {
                        let result = lua.create_table()?;
                        let headers = lua.create_table()?;

                        let response = request.send().await
                            .map_err(|err| LuaError::external(format!("failed to perform request: {err}")))?;

                        result.set("status", response.status().as_u16())?;
                        result.set("is_ok", response.status().is_success())?;
                        result.set("headers", headers.clone())?;

                        for (key, value) in response.headers() {
                            headers.set(key.to_string(), lua.create_string(value.as_bytes())?)?;
                        }

                        Ok::<_, LuaError>((response, result))
                    })?;

                    let mut handles = net_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;
    
                    let mut handle = rand::random::<u32>();

                    while handles.contains_key(&handle) {
                        handle = rand::random::<u32>();
                    }

                    handles.insert(handle, response);

                    header.set("handle", handle)?;

                    Ok(header)
                })?
            },

            net_read: {
                let net_handles = net_handles.clone();

                lua.create_function(move |lua, handle: u32| {
                    let mut handles = net_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;
    
                    let Some(response) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid request handle"));
                    };

                    let chunk = NET_RUNTIME.block_on(async move {
                        response.chunk().await
                            .map_err(|err| {
                                LuaError::external(format!("failed to read body chunk: {err}"))
                            })
                    })?;

                    let Some(chunk) = chunk else {
                        return Ok(LuaNil);
                    };

                    lua.create_sequence_from(chunk)
                        .map(LuaValue::Table)
                })?
            },

            net_close: {
                let net_handles = net_handles.clone();

                lua.create_function(move |_, handle: u32| {
                    net_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?
                        .remove(&handle);

                    Ok(())
                })?
            },

            sync_channel_open: {
                let sync_channels_consumers = sync_channels_consumers.clone();
                let sync_channels_data = sync_channels_data.clone();

                lua.create_function(move |_, key: LuaString| {
                    let mut listeners = sync_channels_data.lock()
                        .map_err(|err| LuaError::external(format!("failed to register channel listeners: {err}")))?;

                    let key = Hash::for_slice(key.as_bytes());
                    let mut handle = rand::random::<u32>();

                    while listeners.contains_key(&handle) {
                        handle = rand::random::<u32>();
                    }

                    let mut consumers = sync_channels_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to register channel consumers: {err}")))?;

                    consumers.entry(key).or_insert_with(HashSet::new);

                    if let Some(listeners) = consumers.get_mut(&key) {
                        listeners.insert(handle);
                    }

                    listeners.insert(handle, (key, VecDeque::new()));

                    Ok(handle)
                })?
            },

            sync_channel_send: {
                let sync_channels_consumers = sync_channels_consumers.clone();
                let sync_channels_data = sync_channels_data.clone();

                lua.create_function(move |_, (handle, message): (u32, LuaValue<'lua>)| {
                    let message = ChannelMessage::from_lua(message)?;

                    let mut listeners = sync_channels_data.lock()
                        .map_err(|err| LuaError::external(format!("failed to read channel listeners: {err}")))?;

                    let Some((key, _)) = listeners.get(&handle) else {
                        return Err(LuaError::external("invalid channel handle"));
                    };

                    let consumers = sync_channels_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to read channel consumers: {err}")))?;

                    let Some(consumers) = consumers.get(key) else {
                        return Err(LuaError::external("invalid channel handle"));
                    };

                    for consumer in consumers {
                        if consumer != &handle {
                            if let Some((_, ref mut data)) = listeners.get_mut(consumer) {
                                data.push_back(message.clone());
                            }
                        }
                    }

                    Ok(())
                })?
            },

            sync_channel_recv: {
                let sync_channels_data = sync_channels_data.clone();

                lua.create_function(move |lua, handle: u32| {
                    let mut listeners = sync_channels_data.lock()
                        .map_err(|err| LuaError::external(format!("failed to read channel listeners: {err}")))?;

                    let Some((_, data)) = listeners.get_mut(&handle) else {
                        return Err(LuaError::external("invalid channel handle"));
                    };

                    match data.pop_front() {
                        Some(message) => Ok((message.to_lua(lua)?, true)),
                        None => Ok((LuaNil, false))
                    }
                })?
            },

            sync_channel_close: {
                let sync_channels_consumers = sync_channels_consumers.clone();
                let sync_channels_data = sync_channels_data.clone();

                lua.create_function(move |_, handle: u32| {
                    let mut consumers = sync_channels_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to read channel consumers: {err}")))?;

                    let mut listeners = sync_channels_data.lock()
                        .map_err(|err| LuaError::external(format!("failed to read channel listeners: {err}")))?;

                    if let Some((hash, _)) = listeners.remove(&handle) {
                        let mut empty = false;

                        if let Some(listeners) = consumers.get_mut(&hash) {
                            listeners.remove(&handle);

                            empty = listeners.is_empty();
                        }

                        if empty {
                            consumers.remove(&hash);
                        }
                    }

                    Ok(())
                })?
            },

            sync_mutex_open: {
                let sync_mutex_consumers = sync_mutex_consumers.clone();

                lua.create_function(move |_, key: LuaString| {
                    let key = Hash::for_slice(key.as_bytes());

                    let mut consumers = sync_mutex_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to register mutex consumers: {err}")))?;

                    let mut handle = rand::random::<u32>();

                    while consumers.contains_key(&handle) {
                        handle = rand::random::<u32>();
                    }

                    consumers.insert(handle, key);

                    Ok(handle)
                })?
            },

            sync_mutex_lock: {
                let sync_mutex_consumers = sync_mutex_consumers.clone();
                let sync_mutex_locks = sync_mutex_locks.clone();

                lua.create_function(move |_, handle: u32| {
                    let key = sync_mutex_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to read mutex consumers: {err}")))?
                        .get(&handle)
                        .copied()
                        .ok_or_else(|| LuaError::external("invalid mutex handle"))?;

                    loop {
                        let mut locks = sync_mutex_locks.lock()
                            .map_err(|err| LuaError::external(format!("failed to read mutex locks: {err}")))?;

                        match locks.get_mut(&key) {
                            Some(lock) => {
                                if lock.is_none() {
                                    *lock = Some(handle);

                                    return Ok(());
                                }
                            }

                            None => {
                                locks.insert(key, Some(handle));

                                return Ok(());
                            }
                        }

                        drop(locks);

                        std::thread::sleep(MUTEX_LOCK_TIMEOUT);
                    }
                })?
            },

            sync_mutex_unlock: {
                let sync_mutex_consumers = sync_mutex_consumers.clone();
                let sync_mutex_locks = sync_mutex_locks.clone();

                lua.create_function(move |_, handle: u32| {
                    let key = sync_mutex_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to read mutex consumers: {err}")))?
                        .get(&handle)
                        .copied()
                        .ok_or_else(|| LuaError::external("invalid mutex handle"))?;

                    let mut locks = sync_mutex_locks.lock()
                        .map_err(|err| LuaError::external(format!("failed to read mutex locks: {err}")))?;

                    if let Some(lock) = locks.get_mut(&key) {
                        if let Some(lock_handle) = lock {
                            if *lock_handle != handle {
                                return Err(LuaError::external("can't unlock mutex locked by another handle"));
                            }

                            *lock = None;
                        }
                    }

                    Ok(())
                })?
            },

            sync_mutex_close: {
                let sync_mutex_consumers = sync_mutex_consumers.clone();
                let sync_mutex_locks = sync_mutex_locks.clone();

                lua.create_function(move |_, handle: u32| {
                    let key = sync_mutex_consumers.lock()
                        .map_err(|err| LuaError::external(format!("failed to read mutex consumers: {err}")))?
                        .remove(&handle);

                    if let Some(key) = key {
                        let mut locks = sync_mutex_locks.lock()
                            .map_err(|err| LuaError::external(format!("failed to read mutex locks: {err}")))?;

                        if let Some(lock) = locks.get_mut(&key) {
                            if let Some(lock_handle) = lock {
                                if *lock_handle == handle {
                                    *lock = None;
                                }
                            }
                        }
                    }

                    Ok(())
                })?
            },

            archive_open: {
                let archive_handles = archive_handles.clone();

                lua.create_function(move |_, (path, format): (LuaString, Option<LuaString>)| {
                    let path = resolve_path(path.to_string_lossy())?;

                    // Parse the archive format.
                    let format = match format {
                        Some(format) => {
                            match format.as_bytes() {
                                b"tar" => ArchiveFormat::Tar,
                                b"zip" => ArchiveFormat::Zip,
                                b"7z"  => ArchiveFormat::Sevenz,

                                _ => return Err(LuaError::external("unsupported archive format"))
                            }
                        }

                        None => ArchiveFormat::from_path(&path)
                            .ok_or_else(|| LuaError::external("unsupported archive format"))?
                    };

                    // Try to open the archive depending on its format.
                    let archive = match format {
                        ArchiveFormat::Tar => TarArchive::open(path)
                            .map_err(|err| LuaError::external(format!("failed to open tar archive: {err}")))
                            .map(Archive::Tar)?,

                        ArchiveFormat::Zip => ZipArchive::open(path)
                            .map_err(|err| LuaError::external(format!("failed to open zip archive: {err}")))
                            .map(Archive::Zip)?,

                        ArchiveFormat::Sevenz => SevenzArchive::open(path)
                            .map_err(|err| LuaError::external(format!("failed to open 7z archive: {err}")))
                            .map(Archive::Sevenz)?,
                    };

                    // Prepare new handle and store the open archive.
                    let mut handles = archive_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                    let mut handle = rand::random::<u32>();

                    while handles.contains_key(&handle) {
                        handle = rand::random::<u32>();
                    }

                    handles.insert(handle, archive);

                    Ok(handle)
                })?
            },

            archive_entries: {
                let archive_handles = archive_handles.clone();

                lua.create_function(move |lua, handle: u32| {
                    let handles = archive_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    // Get archive object using the given handle.
                    let Some(archive) = handles.get(&handle) else {
                        return Err(LuaError::external("invalid archive handle"));
                    };

                    // Get list of archive entries depending on its format.
                    let mut entries = match archive {
                        Archive::Tar(tar) => tar.get_entries()
                            .map_err(|err| LuaError::external(format!("failed to get tar archive entries: {err}")))?,

                        Archive::Zip(zip) => zip.get_entries()
                            .map_err(|err| LuaError::external(format!("failed to get zip archive entries: {err}")))?,

                        Archive::Sevenz(sevenz) => sevenz.get_entries()
                            .map_err(|err| LuaError::external(format!("failed to get 7z archive entries: {err}")))?,
                    };

                    // Prepare the lua output.
                    let entries_table = lua.create_table_with_capacity(entries.len(), 0)?;

                    for entry in entries.drain(..) {
                        let entry_table = lua.create_table()?;

                        entry_table.set("path", entry.path.to_string_lossy())?;
                        entry_table.set("size", entry.size)?;

                        entries_table.push(entry_table)?;
                    }

                    Ok(entries_table)
                })?
            },

            archive_extract: {
                let archive_handles = archive_handles.clone();

                lua.create_function(move |_, (handle, target, progress): (u32, LuaString, Option<LuaFunction>)| {
                    let target = resolve_path(target.to_string_lossy())?;

                    // Start extracting the archive in a background thread depending on its format.
                    let (send, recv) = std::sync::mpsc::channel();

                    let archive_handles = archive_handles.clone();

                    let handle = std::thread::spawn(move || {
                        let handles = archive_handles.lock()
                            .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                        // Get archive object using the given handle.
                        let Some(archive) = handles.get(&handle) else {
                            return Err(LuaError::external("invalid archive handle"));
                        };

                        match archive {
                            Archive::Tar(tar) => tar.extract(target, move |curr, total, diff| {
                                    let _ = send.send((curr, total, diff));
                                })
                                .map_err(|err| LuaError::external(format!("failed to start extracting tar archive: {err}")))?
                                .wait()
                                .map_err(|err| LuaError::external(format!("failed to extract tar archive: {err:?}")))?,

                            Archive::Zip(zip) => zip.extract(target, move |curr, total, diff| {
                                    let _ = send.send((curr, total, diff));
                                })
                                .map_err(|err| LuaError::external(format!("failed to start extracting zip archive: {err}")))?
                                .wait()
                                .map_err(|err| LuaError::external(format!("failed to extract zip archive: {err:?}")))?,

                            Archive::Sevenz(sevenz) => sevenz.extract(target, move |curr, total, diff| {
                                    let _ = send.send((curr, total, diff));
                                })
                                .map_err(|err| LuaError::external(format!("failed to start extracting 7z archive: {err}")))?
                                .wait()
                                .map_err(|err| LuaError::external(format!("failed to extract 7z archive: {err:?}")))?
                        };

                        Ok::<_, LuaError>(())
                    });

                    // Handle extraction progress events.
                    let mut finished = false;

                    while !handle.is_finished() {
                        for (curr, total, diff) in recv.try_iter() {
                            finished = curr == total;

                            if let Some(callback) = &progress {
                                callback.call::<_, ()>((curr, total, diff))?;
                            }
                        }
                    }

                    handle.join().map_err(|err| {
                        LuaError::external(format!("failed to extract archive: {err:?}"))
                    })??;

                    Ok(finished)
                })?
            },

            archive_close: {
                let archive_handles = archive_handles.clone();

                lua.create_function(move |_, handle: u32| {
                    archive_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?
                        .remove(&handle);

                    Ok(())
                })?
            },

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
            },

            process_exec: lua.create_function(|lua, (path, args, env): (LuaString, Option<LuaTable>, Option<LuaTable>)| {
                let path = resolve_path(path.to_string_lossy())?;

                let mut command = Command::new(path);

                    let mut command = command
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());

                // Apply command arguments.
                if let Some(args) = args {
                    for arg in args.sequence_values::<LuaString>() {
                        command = command.arg(arg?.to_string_lossy().to_string());
                    }
                }

                // Apply command environment.
                if let Some(env) = env {
                    for pair in env.pairs::<LuaString, LuaString>() {
                        let (key, value) = pair?;

                        command = command.env(
                            key.to_string_lossy().to_string(),
                            value.to_string_lossy().to_string()
                        );
                    }
                }

                // Execute the command.
                let output = command.output()?;

                // Prepare the output.
                let result = lua.create_table()?;

                result.set("status", output.status.code())?;
                result.set("is_ok", output.status.success())?;
                result.set("stdout", output.stdout)?;
                result.set("stderr", output.stderr)?;

                Ok(result)
            })?,

            process_open: {
                let process_handles = process_handles.clone();

                lua.create_function(move |_, (path, args, env): (LuaString, Option<LuaTable>, Option<LuaTable>)| {
                    let path = resolve_path(path.to_string_lossy())?;

                    let mut command = Command::new(path);

                    let mut command = command
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());

                    // Apply command arguments.
                    if let Some(args) = args {
                        for arg in args.sequence_values::<LuaString>() {
                            command = command.arg(arg?.to_string_lossy().to_string());
                        }
                    }

                    // Apply command environment.
                    if let Some(env) = env {
                        for pair in env.pairs::<LuaString, LuaString>() {
                            let (key, value) = pair?;

                            command = command.env(
                                key.to_string_lossy().to_string(),
                                value.to_string_lossy().to_string()
                            );
                        }
                    }

                    // Start the process and store it.
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                    let mut handle = rand::random::<u32>();

                    while handles.contains_key(&handle) {
                        handle = rand::random::<u32>();
                    }

                    handles.insert(handle, command.spawn()?);

                    Ok(handle)
                })?
            },

            process_stdin: {
                let process_handles = process_handles.clone();

                lua.create_function(move |_, (handle, data): (u32, LuaValue)| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Try to write data to the process's stdin.
                    if let Some(stdin) = &mut process.stdin {
                        stdin.write_all(&get_value_bytes(data)?)?;
                    }

                    Ok(handle)
                })?
            },

            process_stdout: {
                let process_handles = process_handles.clone();

                lua.create_function(move |lua, handle: u32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Read the process's stdout chunk.
                    if let Some(stdout) = &mut process.stdout {
                        let mut buf = [0; PROCESS_READ_CHUNK_LEN];

                        let len = stdout.read(&mut buf)?;

                        return slice_to_table(lua, &buf[..len])
                            .map(LuaValue::Table);
                    }

                    Ok(LuaNil)
                })?
            },

            process_stderr: {
                let process_handles = process_handles.clone();

                lua.create_function(move |lua, handle: u32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Read the process's stderr chunk.
                    if let Some(stderr) = &mut process.stderr {
                        let mut buf = [0; PROCESS_READ_CHUNK_LEN];

                        let len = stderr.read(&mut buf)?;

                        return slice_to_table(lua, &buf[..len])
                            .map(LuaValue::Table);
                    }

                    Ok(LuaNil)
                })?
            },

            process_kill: {
                let process_handles = process_handles.clone();

                lua.create_function(move |_, handle: u32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Kill the process and remove its handle. 
                    process.kill()?;
                    handles.remove(&handle);

                    Ok(())
                })?
            },

            process_wait: {
                let process_handles = process_handles.clone();

                lua.create_function(move |lua, handle: u32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.remove(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Wait until the process has finished.
                    let output = process.wait_with_output()?;

                    // Prepare lua result.
                    let result = lua.create_table()?;

                    result.set("status", output.status.code())?;
                    result.set("is_ok", output.status.success())?;
                    result.set("stdout", output.stdout)?;
                    result.set("stderr", output.stderr)?;

                    Ok(result)
                })?
            },

            process_finished: {
                let process_handles = process_handles.clone();

                lua.create_function(move |_, handle: u32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    Ok(process.try_wait()?.is_some())
                })?
            }
        })
    }

    /// Create new environment for the v1 modules standard.
    /// 
    /// If `extended_privileges` enabled, then the result
    /// table will contain functions that can escape the
    /// default sandbox and execute code on the host machine.
    pub fn create_env(&self, extended_privileges: bool) -> Result<LuaTable<'lua>, EngineError> {
        let env = self.lua.create_table()?;

        env.set("clone", self.clone.clone())?;

        let str = self.lua.create_table()?;
        let path = self.lua.create_table()?;
        let fs = self.lua.create_table()?;
        let net = self.lua.create_table()?;

        let sync = self.lua.create_table()?;
        let sync_channel = self.lua.create_table()?;
        let sync_mutex = self.lua.create_table()?;

        let archive = self.lua.create_table()?;
        let hash = self.lua.create_table()?;

        env.set("str", str.clone())?;
        env.set("path", path.clone())?;
        env.set("fs", fs.clone())?;
        env.set("net", net.clone())?;

        env.set("sync", sync.clone())?;
        sync.set("channel", sync_channel.clone())?;
        sync.set("mutex", sync_mutex.clone())?;

        env.set("archive", archive.clone())?;
        env.set("hash", hash.clone())?;

        // String API

        str.set("to_bytes", self.str_to_bytes.clone())?;
        str.set("from_bytes", self.str_from_bytes.clone())?;
        str.set("encode", self.str_encode.clone())?;
        str.set("decode", self.str_decode.clone())?;

        // Paths API

        path.set("temp_dir", self.path_temp_dir.clone())?;
        path.set("module_dir", self.path_module_dir.clone())?;
        path.set("persist_dir", self.path_persist_dir.clone())?;
        path.set("normalize", self.path_normalize.clone())?;
        path.set("join", self.path_join.clone())?;
        path.set("parts", self.path_parts.clone())?;
        path.set("parent", self.path_parent.clone())?;
        path.set("file_name", self.path_file_name.clone())?;
        path.set("exists", self.path_exists.clone())?;
        path.set("accessible", self.path_accessible.clone())?;

        // IO API

        fs.set("exists", self.fs_exists.clone())?;
        fs.set("metadata", self.fs_metadata.clone())?;
        fs.set("copy", self.fs_copy.clone())?;
        fs.set("move", self.fs_move.clone())?;
        fs.set("remove", self.fs_remove.clone())?;
        fs.set("open", self.fs_open.clone())?;
        fs.set("seek", self.fs_seek.clone())?;
        fs.set("read", self.fs_read.clone())?;
        fs.set("write", self.fs_write.clone())?;
        fs.set("flush", self.fs_flush.clone())?;
        fs.set("close", self.fs_close.clone())?;

        fs.set("read_file", self.fs_read_file.clone())?;
        fs.set("write_file", self.fs_write_file.clone())?;
        fs.set("remove_file", self.fs_remove_file.clone())?;
        fs.set("create_dir", self.fs_create_dir.clone())?;
        fs.set("read_dir", self.fs_read_dir.clone())?;
        fs.set("remove_dir", self.fs_remove_dir.clone())?;

        // Network API

        net.set("fetch", self.net_fetch.clone())?;
        net.set("open", self.net_open.clone())?;
        net.set("read", self.net_read.clone())?;
        net.set("close", self.net_close.clone())?;

        // Sync API - Channels

        sync_channel.set("open", self.sync_channel_open.clone())?;
        sync_channel.set("send", self.sync_channel_send.clone())?;
        sync_channel.set("recv", self.sync_channel_recv.clone())?;
        sync_channel.set("close", self.sync_channel_close.clone())?;

        // Sync API - Mutex

        sync_mutex.set("open", self.sync_mutex_open.clone())?;
        sync_mutex.set("lock", self.sync_mutex_lock.clone())?;
        sync_mutex.set("unlock", self.sync_mutex_unlock.clone())?;
        sync_mutex.set("close", self.sync_mutex_close.clone())?;

        // Archives API

        archive.set("open", self.archive_open.clone())?;
        archive.set("entries", self.archive_entries.clone())?;
        archive.set("extract", self.archive_extract.clone())?;
        archive.set("close", self.archive_close.clone())?;

        // Hashes API

        hash.set("calc", self.hash_calc.clone())?;
        hash.set("builder", self.hash_builder.clone())?;
        hash.set("write", self.hash_write.clone())?;
        hash.set("finalize", self.hash_finalize.clone())?;

        // Extended privileges

        if extended_privileges {
            let process = self.lua.create_table()?;

            env.set("process", process.clone())?;

            // Process API

            process.set("exec", self.process_exec.clone())?;
            process.set("open", self.process_open.clone())?;
            process.set("stdin", self.process_stdin.clone())?;
            process.set("stdout", self.process_stdout.clone())?;
            process.set("stderr", self.process_stderr.clone())?;
            process.set("wait", self.process_wait.clone())?;
            process.set("kill", self.process_kill.clone())?;
            process.set("finished", self.process_finished.clone())?;
        }

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_bytes() -> anyhow::Result<()> {
        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        assert_eq!(standard.str_to_bytes.call::<_, Vec<u8>>("abc")?, &[97, 98, 99]);
        assert_eq!(standard.str_to_bytes.call::<_, Vec<u8>>(0.5)?, &[63, 224, 0, 0, 0, 0, 0, 0]);
        assert_eq!(standard.str_to_bytes.call::<_, Vec<u8>>(vec![1, 2, 3])?, &[1, 2, 3]);

        assert_eq!(standard.str_to_bytes.call::<_, Vec<u8>>("абоба")?, &[208, 176, 208, 177, 208, 190, 208, 177, 208, 176]);
        assert_eq!(standard.str_to_bytes.call::<_, Vec<u8>>(("абоба", "cp1251"))?, &[224, 225, 238, 225, 224]);

        assert_eq!(standard.str_from_bytes.call::<_, LuaString>(vec![97, 98, 99])?, b"abc");

        assert_eq!(standard.str_from_bytes.call::<_, LuaString>(vec![208, 176, 208, 177, 208, 190, 208, 177, 208, 176])?, "абоба");
        assert_eq!(standard.str_from_bytes.call::<_, LuaString>((vec![224, 225, 238, 225, 224], "cp1251"))?, "абоба");

        Ok(())
    }

    #[test]
    fn str_encodings() -> anyhow::Result<()> {
        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        let encodings = [
            ("hex",                  "48656c6c6f2c20576f726c6421"),
            ("base16",               "48656c6c6f2c20576f726c6421"),
            ("base32",               "jbswy3dpfqqfo33snrscc==="),
            ("base32/pad",           "jbswy3dpfqqfo33snrscc==="),
            ("base32/nopad",         "jbswy3dpfqqfo33snrscc"),
            ("base32/hex-pad",       "91imor3f5gg5erridhi22==="),
            ("base32/hex-nopad",     "91imor3f5gg5erridhi22"),
            ("base64",               "SGVsbG8sIFdvcmxkIQ=="),
            ("base64/pad",           "SGVsbG8sIFdvcmxkIQ=="),
            // ("base64/nopad",         "SGVsbG8sIFdvcmxkIQ"),
            ("base64/urlsafe-pad",   "SGVsbG8sIFdvcmxkIQ=="),
            // ("base64/urlsafe-nopad", "SGVsbG8sIFdvcmxkIQ")
        ];

        for (name, value) in encodings {
            let encoded = standard.str_encode.call::<_, LuaString>(("Hello, World!", name))?;
            let decoded = standard.str_decode.call::<_, Vec<u8>>((value, name))?;

            assert_eq!(encoded, value);
            assert_eq!(decoded, b"Hello, World!");
        }

        let table = lua.create_table()?;

        table.set("hello", "world")?;

        let encodings = [
            ("json", "{\"hello\":\"world\"}")
        ];

        for (name, value) in encodings {
            let encoded = standard.str_encode.call::<_, LuaString>((table.clone(), name))?;
            let decoded = standard.str_decode.call::<_, LuaTable>((value, name))?;

            assert_eq!(encoded, value);
            assert_eq!(decoded.get::<_, LuaString>("hello")?, "world");
        }

        Ok(())
    }

    #[test]
    fn path_actions() -> anyhow::Result<()> {
        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        assert_eq!(standard.path_normalize.call::<_, String>("/")?, "/");
        assert_eq!(standard.path_normalize.call::<_, String>("a/b/c")?, "a/b/c");
        assert_eq!(standard.path_normalize.call::<_, String>("/a/b/c")?, "/a/b/c");
        assert_eq!(standard.path_normalize.call::<_, String>("a/./c")?, "a/c");
        assert_eq!(standard.path_normalize.call::<_, String>("a/../c")?, "c");
        assert_eq!(standard.path_normalize.call::<_, String>("a/../c/./")?, "c");
        assert_eq!(standard.path_normalize.call::<_, String>("./a//\\./../b")?, "b");
        assert_eq!(standard.path_normalize.call::<_, String>(" ")?, " "); // space is a correct entry name
        assert_eq!(standard.path_normalize.call::<_, Option<String>>("")?, None); // entry name cannot be empty
        assert_eq!(standard.path_normalize.call::<_, Option<String>>(".")?, None); // we do not support relative paths
        assert_eq!(standard.path_normalize.call::<_, Option<String>>("..")?, None);
        assert_eq!(standard.path_normalize.call::<_, Option<String>>("./..")?, None);
        assert_eq!(standard.path_normalize.call::<_, Option<String>>("a/..")?, None);

        assert_eq!(standard.path_join.call::<_, String>(vec!["a", "b", "c"])?, "a/b/c");
        assert_eq!(standard.path_join.call::<_, String>(vec!["/", "a", "b", "c"])?, "/a/b/c");
        assert_eq!(standard.path_join.call::<_, String>(vec!["a", "..", "b"])?, "b");
        assert_eq!(standard.path_join.call::<_, String>(vec![".", "a", ".", "b"])?, "a/b");
        assert_eq!(standard.path_join.call::<_, Option<String>>(vec![""])?, None);
        assert_eq!(standard.path_join.call::<_, Option<String>>(vec!["."])?, None);
        assert_eq!(standard.path_join.call::<_, Option<String>>(vec![".."])?, None);
        assert_eq!(standard.path_join.call::<_, Option<String>>(vec![".", ".."])?, None);
        assert_eq!(standard.path_join.call::<_, Option<String>>(vec!["a", ".."])?, None);

        assert_eq!(standard.path_parts.call::<_, Vec<String>>("a/b/c")?, &["a", "b", "c"]);
        assert_eq!(standard.path_parts.call::<_, Vec<String>>("a/./c")?, &["a", "c"]);
        assert_eq!(standard.path_parts.call::<_, Vec<String>>("a/./c/..")?, &["a"]);
        assert_eq!(standard.path_parts.call::<_, Vec<String>>("\\a/b/// /c")?, &["a", "b", " ", "c"]);
        assert_eq!(standard.path_parts.call::<_, Option<Vec<String>>>("")?, None);
        assert_eq!(standard.path_parts.call::<_, Option<Vec<String>>>(".")?, None);
        assert_eq!(standard.path_parts.call::<_, Option<Vec<String>>>("..")?, None);
        assert_eq!(standard.path_parts.call::<_, Option<Vec<String>>>("./..")?, None);
        assert_eq!(standard.path_parts.call::<_, Option<Vec<String>>>("a/..")?, None);

        assert_eq!(standard.path_parent.call::<_, String>("a/b/c")?, "a/b");
        assert_eq!(standard.path_parent.call::<_, String>("/a/b/c")?, "/a/b");
        assert_eq!(standard.path_parent.call::<_, String>("a\\./b")?, "a");
        assert_eq!(standard.path_parent.call::<_, Option<Vec<String>>>("a")?, None);
        assert_eq!(standard.path_parent.call::<_, Option<Vec<String>>>("a/.")?, None);
        assert_eq!(standard.path_parent.call::<_, Option<Vec<String>>>("a/../b")?, None);

        assert_eq!(standard.path_file_name.call::<_, String>("/")?, "/");
        assert_eq!(standard.path_file_name.call::<_, String>("a")?, "a");
        assert_eq!(standard.path_file_name.call::<_, String>("a/b/c")?, "c");
        assert_eq!(standard.path_file_name.call::<_, String>("/a/b/c")?, "c");
        assert_eq!(standard.path_file_name.call::<_, String>("a\\./b")?, "b");
        assert_eq!(standard.path_file_name.call::<_, Option<Vec<String>>>(".")?, None);
        assert_eq!(standard.path_file_name.call::<_, Option<Vec<String>>>("a/..")?, None);

        Ok(())
    }

    #[test]
    fn file_handle() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-v1-file-handle-test");

        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        let path = path.to_string_lossy().to_string();

        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        assert!(!standard.fs_exists.call::<_, bool>(path.clone())?);
        assert!(standard.fs_open.call::<_, u64>(path.clone()).is_err());

        let options = lua.create_table()?;

        options.set("read", true)?;
        options.set("write", true)?;
        options.set("create", true)?;

        let handle = standard.fs_open.call::<_, u64>((path.clone(), options))?;

        assert_eq!(standard.fs_read.call::<_, Vec<u8>>(handle)?.len(), 0);

        standard.fs_write.call::<_, ()>((handle, b"Hello, ".to_vec()))?;
        standard.fs_write.call::<_, ()>((handle, b"World!".to_vec()))?;
        standard.fs_flush.call::<_, ()>(handle)?;

        standard.fs_seek.call::<_, ()>((handle, 0))?;

        assert_eq!(standard.fs_read.call::<_, Vec<u8>>(handle)?, b"Hello, World!");

        standard.fs_seek.call::<_, ()>((handle, 0))?;
        standard.fs_write.call::<_, ()>((handle, b"Amogus".to_vec()))?;
        standard.fs_flush.call::<_, ()>(handle)?;

        standard.fs_seek.call::<_, ()>((handle, 0))?;

        assert_eq!(standard.fs_read.call::<_, Vec<u8>>(handle)?, b"Amogus World!");

        standard.fs_seek.call::<_, ()>((handle, -6))?;
        standard.fs_write.call::<_, ()>((handle, b"Amogus".to_vec()))?;
        standard.fs_flush.call::<_, ()>(handle)?;

        standard.fs_seek.call::<_, ()>((handle, 0))?;

        assert_eq!(standard.fs_read.call::<_, Vec<u8>>(handle)?, b"Amogus Amogus");

        standard.fs_seek.call::<_, ()>((handle, 0))?;
        standard.fs_write.call::<_, ()>((handle, b"Sugoma".to_vec()))?;

        assert_eq!(standard.fs_read.call::<_, Vec<u8>>(handle)?, b" Amogus");

        standard.fs_flush.call::<_, ()>(handle)?;
        standard.fs_seek.call::<_, ()>((handle, 0))?;

        assert_eq!(standard.fs_read.call::<_, Vec<u8>>(handle)?, b"Sugoma Amogus");
        assert_eq!(standard.fs_read.call::<_, Vec<u8>>((handle, 3, 7))?, b"oma Amo");
        assert_eq!(standard.fs_read.call::<_, Vec<u8>>(handle)?, b"gus");
        assert_eq!(standard.fs_read.call::<_, Vec<u8>>((handle, -6))?, b"Amogus");

        standard.fs_write.call::<_, ()>((handle, b"Mogusa".to_vec(), 0))?;
        standard.fs_write.call::<_, ()>((handle, b"Susoma".to_vec(), 7))?;

        assert_eq!(standard.fs_read.call::<_, Vec<u8>>((handle, 0))?, b"Mogusa Susoma");

        standard.fs_close.call::<_, ()>(handle)?;

        assert!(standard.fs_read.call::<_, Vec<u8>>(handle).is_err());

        Ok(())
    }

    #[test]
    fn file_actions() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-v1-file-actions-test");

        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        let path = path.to_string_lossy().to_string();

        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        assert!(!standard.fs_exists.call::<_, bool>(path.clone())?);

        standard.fs_write_file.call::<_, ()>((path.clone(), vec![1, 2, 3]))?;

        assert!(standard.fs_exists.call::<_, bool>(path.clone())?);

        let metadata = standard.fs_metadata.call::<_, LuaTable>(path.clone())?;

        assert_eq!(metadata.get::<_, u32>("length")?, 3);
        assert_eq!(metadata.get::<_, String>("type")?, "file");
        assert!(metadata.get::<_, bool>("is_accessible")?);

        assert_eq!(standard.fs_read_file.call::<_, Vec<u8>>(path.clone())?, &[1, 2, 3]);

        assert!(standard.fs_copy.call::<_, ()>((format!("{path}123"), format!("{path}456"))).is_err());
        assert!(standard.fs_copy.call::<_, ()>((path.clone(), path.clone())).is_err());

        standard.fs_copy.call::<_, ()>((path.clone(), format!("{path}_copy")))?;

        assert!(standard.fs_exists.call::<_, bool>(format!("{path}_copy"))?);

        standard.fs_remove_file.call::<_, ()>(path.clone())?;

        assert!(!standard.fs_exists.call::<_, bool>(path.clone())?);

        standard.fs_move.call::<_, ()>((format!("{path}_copy"), path.clone()))?;

        assert!(!standard.fs_exists.call::<_, bool>(format!("{path}_copy"))?);
        assert!(standard.fs_exists.call::<_, bool>(path.clone())?);

        standard.fs_remove.call::<_, ()>(path.clone())?;

        assert!(!standard.fs_exists.call::<_, bool>(path.clone())?);

        Ok(())
    }

    #[tokio::test]
    async fn folder_actions() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-v1-folder-actions-test");
        let dxvk_path = std::env::temp_dir().join(".agl-v1-folder-actions-test-dxvk.tar.gz");

        if path.exists() {
            std::fs::remove_dir_all(&path)?;
        }

        let path = path.to_string_lossy().to_string();

        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        assert!(!standard.fs_exists.call::<_, bool>(path.clone())?);

        standard.fs_create_dir.call::<_, ()>(path.clone())?;

        assert!(standard.fs_exists.call::<_, bool>(path.clone())?);

        let metadata = standard.fs_metadata.call::<_, LuaTable>(path.clone())?;

        assert_eq!(metadata.get::<_, String>("type")?, "folder");
        assert!(metadata.get::<_, bool>("is_accessible")?);

        if !dxvk_path.exists() {
            Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz")
                .map_err(|err| anyhow::anyhow!(err.to_string()))?
                .with_output_file(&dxvk_path)
                .download(|_, _, _| {})
                .await
                .map_err(|err| anyhow::anyhow!(err.to_string()))?
                .wait()
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;
        }

        super::archive_extract(dxvk_path, &path, |_, _, _| {})?;

        let path = format!("{path}/dxvk-2.4");

        assert_eq!(Hash::for_entry(&path)?, Hash(15040088835594252178));

        let entries = standard.fs_read_dir.call::<_, LuaTable>(path.clone())?;

        assert_eq!(entries.len()?, 2);

        for _ in 0..2 {
            let entry = entries.pop::<LuaTable>()?;

            assert!(["x32", "x64"].contains(&entry.get::<_, String>("name")?.as_str()));
            assert!(std::fs::exists(&entry.get::<_, String>("path")?)?);
            assert_eq!(entry.get::<_, String>("type")?, "folder");
        }

        assert!(!standard.fs_exists.call::<_, bool>(format!("{path}_copy"))?);

        standard.fs_copy.call::<_, ()>((path.clone(), format!("{path}_copy")))?;

        assert!(standard.fs_exists.call::<_, bool>(format!("{path}_copy"))?);

        assert!(standard.fs_remove_file.call::<_, ()>(path.clone()).is_err());

        standard.fs_remove_dir.call::<_, ()>(path.clone())?;

        assert!(!standard.fs_exists.call::<_, bool>(path.clone())?);

        standard.fs_move.call::<_, ()>((format!("{path}_copy"), path.clone()))?;

        assert!(!standard.fs_exists.call::<_, bool>(format!("{path}_copy"))?);
        assert!(standard.fs_exists.call::<_, bool>(path.clone())?);

        assert_eq!(Hash::for_entry(&path)?, Hash(15040088835594252178));

        Ok(())
    }

    #[test]
    fn net_fetch() -> anyhow::Result<()> {
        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        let response = standard.net_fetch.call::<_, LuaTable>(
            "https://raw.githubusercontent.com/an-anime-team/anime-games-launcher/refs/heads/next/tests/packages/1/package.json"
        )?;

        assert_eq!(response.get::<_, u16>("status")?, 200);
        assert!(response.get::<_, bool>("is_ok")?);
        assert_eq!(Hash::for_slice(&response.get::<_, Vec<u8>>("body")?), Hash(9442626994218140953));

        Ok(())
    }

    #[test]
    fn net_read() -> anyhow::Result<()> {
        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        let header = standard.net_open.call::<_, LuaTable>(
            "https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz"
        )?;

        assert_eq!(header.get::<_, u16>("status")?, 200);
        assert!(header.get::<_, bool>("is_ok")?);

        let handle = header.get::<_, u32>("handle")?;

        let mut body_len = 0;

        while let Some(chunk) = standard.net_read.call::<_, Option<Vec<u8>>>(handle)? {
            body_len += chunk.len();
        }

        assert_eq!(body_len, 9215513);

        Ok(())
    }

    #[test]
    fn sync_channels() -> anyhow::Result<()> {
        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        assert!(standard.sync_channel_send.call::<_, ()>((0, String::new())).is_err());
        assert!(standard.sync_channel_recv.call::<_, Option<String>>(0).is_err());

        let a = standard.sync_channel_open.call::<_, u32>("test")?;
        let b = standard.sync_channel_open.call::<_, u32>("test")?;

        assert_eq!(standard.sync_channel_recv.call::<_, Option<String>>(a)?, None);
        assert_eq!(standard.sync_channel_recv.call::<_, Option<String>>(b)?, None);

        standard.sync_channel_send.call::<_, ()>((a, String::from("Message 1")))?;
        standard.sync_channel_send.call::<_, ()>((a, String::from("Message 2")))?;

        let c = standard.sync_channel_open.call::<_, u32>("test")?;

        assert_eq!(standard.sync_channel_recv.call::<_, Option<String>>(a)?, None);
        assert_eq!(standard.sync_channel_recv.call::<_, Option<String>>(c)?, None);
        assert_eq!(standard.sync_channel_recv.call::<_, String>(b)?, "Message 1");
        assert_eq!(standard.sync_channel_recv.call::<_, String>(b)?, "Message 2");
        assert_eq!(standard.sync_channel_recv.call::<_, Option<String>>(b)?, None);

        standard.sync_channel_send.call::<_, ()>((a, String::from("Message 3")))?;

        assert_eq!(standard.sync_channel_recv.call::<_, Option<String>>(a)?, None);
        assert_eq!(standard.sync_channel_recv.call::<_, String>(b)?, "Message 3");
        assert_eq!(standard.sync_channel_recv.call::<_, String>(c)?, "Message 3");
        assert_eq!(standard.sync_channel_recv.call::<_, Option<String>>(b)?, None);
        assert_eq!(standard.sync_channel_recv.call::<_, Option<String>>(c)?, None);

        standard.sync_channel_send.call::<_, ()>((a, true))?;
        standard.sync_channel_send.call::<_, ()>((a, 0.5))?;
        standard.sync_channel_send.call::<_, ()>((a, -17))?;
        standard.sync_channel_send.call::<_, ()>((a, vec![1, 2, 3]))?;
        standard.sync_channel_send.call::<_, ()>((a, vec!["Hello", "World"]))?;
        standard.sync_channel_send.call::<_, ()>((a, vec![vec![1, 2], vec![3, 4]]))?;

        assert_eq!(standard.sync_channel_recv.call::<_, Option<_>>(b)?, Some(true));
        assert_eq!(standard.sync_channel_recv.call::<_, Option<_>>(b)?, Some(0.5));
        assert_eq!(standard.sync_channel_recv.call::<_, Option<_>>(b)?, Some(-17));
        assert_eq!(standard.sync_channel_recv.call::<_, Option<_>>(b)?, Some(vec![1, 2, 3]));
        assert_eq!(standard.sync_channel_recv.call::<_, Option<_>>(b)?, Some(vec![String::from("Hello"), String::from("World")]));
        assert_eq!(standard.sync_channel_recv.call::<_, Option<_>>(b)?, Some(vec![vec![1, 2], vec![3, 4]]));
        assert_eq!(standard.sync_channel_recv.call::<_, Option<String>>(b)?, None);

        standard.sync_channel_close.call::<_, ()>(a)?;
        standard.sync_channel_close.call::<_, ()>(b)?;
        standard.sync_channel_close.call::<_, ()>(c)?;

        assert!(standard.sync_channel_send.call::<_, ()>((a, String::new())).is_err());
        assert!(standard.sync_channel_recv.call::<_, Option<String>>(a).is_err());

        assert!(standard.sync_channel_send.call::<_, ()>((b, String::new())).is_err());
        assert!(standard.sync_channel_recv.call::<_, Option<String>>(b).is_err());

        assert!(standard.sync_channel_send.call::<_, ()>((c, String::new())).is_err());
        assert!(standard.sync_channel_recv.call::<_, Option<String>>(c).is_err());

        Ok(())
    }

    #[tokio::test]
    async fn archive_entries() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-v1-archive-test-dxvk.tar.gz");

        if !path.exists() {
            Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz").unwrap()
                .with_output_file(&path)
                .download(|_, _, _| {})
                .await.unwrap()
                .wait().unwrap();
        }

        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        assert!(standard.archive_entries.call::<_, LuaTable>(0).is_err());
        assert!(standard.archive_extract.call::<_, LuaTable>(0).is_err());

        let handle = standard.archive_open.call::<_, u32>(path.to_string_lossy())?;
        let entries = standard.archive_entries.call::<_, LuaTable>(handle)?;

        assert_eq!(entries.len()?, 13);

        let mut total_size = 0;
        let mut has_path = false;

        for entry in entries.sequence_values::<LuaTable>() {
            let entry = entry?;
        
            let path = entry.get::<_, String>("path")?;
            let size = entry.get::<_, u64>("size")?;

            total_size += size;

            if path == "dxvk-2.4/x64/d3d10core.dll" {
                has_path = true;
            }
        }

        assert_eq!(total_size, 25579660);
        assert!(has_path);

        standard.archive_close.call::<_, ()>(handle)?;

        assert!(standard.archive_entries.call::<_, LuaTable>(handle).is_err());
        assert!(standard.archive_extract.call::<_, LuaTable>(handle).is_err());

        Ok(())
    }

    #[tokio::test]
    async fn archive_extract() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-v1-archive-test");
        let dxvk_path = std::env::temp_dir().join(".agl-v1-archive-test-dxvk.tar.gz");

        if !dxvk_path.exists() {
            Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz").unwrap()
                .with_output_file(&dxvk_path)
                .download(|_, _, _| {})
                .await.unwrap()
                .wait().unwrap();
        }

        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        assert!(standard.archive_entries.call::<_, LuaTable>(0).is_err());
        assert!(standard.archive_extract.call::<_, LuaTable>(0).is_err());

        let handle = standard.archive_open.call::<_, u32>(dxvk_path.to_string_lossy())?;
        let result = standard.archive_extract.call::<_, bool>((handle, path.to_string_lossy()))?;

        assert!(result);
        assert_eq!(Hash::for_entry(path)?, Hash(17827013605004440863));

        standard.archive_close.call::<_, ()>(handle)?;

        assert!(standard.archive_entries.call::<_, LuaTable>(handle).is_err());
        assert!(standard.archive_extract.call::<_, LuaTable>(handle).is_err());

        Ok(())
    }

    #[test]
    fn hash_calc() -> anyhow::Result<()> {
        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(0.5)?,             &[120, 18, 28, 179, 226, 204, 30, 109]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(-17)?,             &[176, 134, 88, 13, 238, 58, 194, 165]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>("Hello, World!")?, &[46, 194, 87, 41, 102, 208, 6, 253]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(vec![1, 2, 3])?,   &[142, 143, 212, 110, 1, 110, 210, 66]);

        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "seahash"))?,  &[46, 194, 87, 41, 102, 208, 6, 253]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "crc32"))?,    &[236, 74, 195, 208]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "crc32c"))?,   &[77, 85, 16, 104]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "xxh32"))?,    &[64, 7, 222, 80]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "xxh64"))?,    &[196, 154, 172, 248, 8, 15, 228, 127]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "xxh3-64"))?,  &[96, 65, 93, 95, 97, 102, 2, 170]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "xxh3-128"))?, &[83, 29, 242, 132, 68, 71, 221, 80, 119, 219, 3, 132, 44, 215, 83, 149]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "md5"))?,      &[101, 168, 226, 125, 136, 121, 40, 56, 49, 182, 100, 189, 139, 127, 10, 212]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "sha1"))?,     &[10, 10, 159, 42, 103, 114, 148, 37, 87, 171, 83, 85, 215, 106, 244, 66, 248, 246, 94, 1]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "sha2-224"))?, &[114, 162, 61, 250, 65, 27, 166, 253, 224, 29, 191, 171, 243, 176, 10, 112, 156, 147, 235, 242, 115, 220, 41, 226, 216, 178, 97, 255]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "sha2-256"))?, &[223, 253, 96, 33, 187, 43, 213, 176, 175, 103, 98, 144, 128, 158, 195, 165, 49, 145, 221, 129, 199, 247, 10, 75, 40, 104, 138, 54, 33, 130, 152, 111]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "sha2-384"))?, &[84, 133, 204, 155, 51, 101, 180, 48, 93, 251, 78, 131, 55, 224, 165, 152, 165, 116, 248, 36, 43, 241, 114, 137, 224, 221, 108, 32, 163, 205, 68, 160, 137, 222, 22, 171, 74, 179, 8, 246, 62, 68, 177, 23, 14, 181, 245, 21]);
        assert_eq!(standard.hash_calc.call::<_, Vec<u8>>(("Hello, World!", "sha2-512"))?, &[55, 77, 121, 74, 149, 205, 207, 216, 179, 89, 147, 24, 95, 239, 155, 163, 104, 241, 96, 216, 218, 244, 50, 208, 139, 169, 241, 237, 30, 90, 190, 108, 198, 146, 145, 224, 250, 47, 224, 0, 106, 82, 87, 14, 241, 140, 25, 222, 244, 230, 23, 195, 60, 229, 46, 240, 166, 229, 251, 227, 24, 203, 3, 135]);

        Ok(())
    }

    #[test]
    fn hash_builder() -> anyhow::Result<()> {
        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

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
            let hasher = standard.hash_builder.call::<_, u32>(name)?;

            standard.hash_write.call::<_, ()>((hasher, "Hello"))?;
            standard.hash_write.call::<_, ()>((hasher, ", "))?;
            standard.hash_write.call::<_, ()>((hasher, "World"))?;
            standard.hash_write.call::<_, ()>((hasher, "!"))?;

            assert_eq!(standard.hash_finalize.call::<_, Vec<u8>>(hasher)?, hash);
        }

        Ok(())
    }

    #[test]
    fn process_exec() -> anyhow::Result<()> {
        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        let output = standard.process_exec.call::<_, LuaTable>((
            "bash", ["-c", "echo $TEST"],
            HashMap::from([
                ("TEST", "Hello, World!")
            ])
        ))?;

        assert_eq!(output.get::<_, i32>("status")?, 0);
        assert!(output.get::<_, bool>("is_ok")?);
        assert_eq!(output.get::<_, Vec<u8>>("stdout")?, b"Hello, World!\n");

        Ok(())
    }

    #[test]
    fn process_open() -> anyhow::Result<()> {
        let lua = Lua::new();
        let standard = Standard::new(&lua)?;

        let handle = standard.process_open.call::<_, u32>((
            "bash", ["-c", "echo $TEST"],
            HashMap::from([
                ("TEST", "Hello, World!")
            ])
        ))?;

        while !standard.process_finished.call::<_, bool>(handle)? {
            std::thread::sleep(Duration::from_millis(100));
        }

        assert_eq!(standard.process_stdout.call::<_, Vec<u8>>(handle)?, b"Hello, World!\n");

        let output = standard.process_wait.call::<_, LuaTable>(handle)?;

        assert_eq!(output.get::<_, i32>("status")?, 0);
        assert!(output.get::<_, bool>("is_ok")?);
        assert!(output.get::<_, Vec<u8>>("stdout")?.is_empty());

        Ok(())
    }
}
