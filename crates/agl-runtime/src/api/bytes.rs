// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-runtime
// Copyright (C) 2026  Nikita Podvirnyi <krypt0nn@vk.com>
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

use std::io::{Read, Seek, SeekFrom};

use mlua::prelude::*;

pub const BYTES_READ_CHUNK_SIZE: usize = 8192;

/// Immutable slice of binary data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bytes {
    buf: Box<[u8]>,
    pos: usize,
    len: usize
}

impl Bytes {
    #[inline]
    pub const fn new(buf: Box<[u8]>) -> Self {
        Self {
            pos: 0,
            len: buf.len(),
            buf
        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub const fn as_slice(&self) -> &[u8] {
        &self.buf
    }
}

impl Read for Bytes {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // We haven't reached the end of the buffer yet.
        if self.pos < self.len {
            // Amount of bytes to read.
            let n = self.len.saturating_sub(self.pos)
                .min(BYTES_READ_CHUNK_SIZE);

            // Should never happen.
            if n == 0 {
                return Ok(0);
            }

            let new_pos = self.pos + n;

            buf.copy_from_slice(&self.buf[self.pos..new_pos]);

            self.pos = new_pos;

            Ok(n)
        }

        // We've reached the end of the buffer.
        else {
            Ok(0)
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let Some(n) = self.len.checked_sub(self.pos) else {
            return Ok(0);
        };

        buf.copy_from_slice(&self.buf[self.pos..]);

        self.pos = self.len;

        Ok(n)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        let n = buf.len();
        let new_pos = self.pos + n;

        if new_pos > self.len {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }

        buf.copy_from_slice(&self.buf[self.pos..new_pos]);

        self.pos = new_pos;

        Ok(())
    }
}

impl Seek for Bytes {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(pos) => pos,

            SeekFrom::End(pos) => {
                let Some(pos) = (self.len as i64).checked_sub(pos) else {
                    return Err(std::io::ErrorKind::UnexpectedEof.into());
                };

                // Don't allow lower than 0 position.
                if pos < 0 {
                    return Err(std::io::ErrorKind::UnexpectedEof.into());
                }

                pos as u64
            }

            SeekFrom::Current(mut pos) => {
                pos += self.pos as i64;

                // Don't allow lower than 0 position.
                if pos < 0 {
                    return Err(std::io::ErrorKind::UnexpectedEof.into());
                }

                pos as u64
            }
        };

        // Convert u64 position back to usize.
        let Ok(new_pos) = usize::try_from(new_pos) else {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        };

        // Since our bytes is immutable - we shouldn't allow out of bounds
        // seeking.
        if new_pos > self.len {
            return Err(std::io::ErrorKind::UnexpectedEof.into());
        }

        self.pos = new_pos;

        Ok(new_pos as u64)
    }

    #[inline]
    fn rewind(&mut self) -> std::io::Result<()> {
        self.pos = 0;

        Ok(())
    }

    #[inline]
    fn stream_position(&mut self) -> std::io::Result<u64> {
        Ok(self.pos as u64)
    }
}

impl FromLua for Bytes {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::String(str) => {
                Ok(Self::new(str.as_bytes().to_vec().into_boxed_slice()))
            }

            LuaValue::Table(table) => {
                let bytes = table.sequence_values::<u8>()
                    .collect::<Result<Box<[u8]>, LuaError>>()?;

                Ok(Self::new(bytes))
            }

            LuaValue::UserData(object) if object.get::<Option<LuaFunction>>("as_table")?.is_some() => {
                let bytes = object.call_method::<LuaTable>("as_table", ())?
                    .sequence_values::<u8>()
                    .collect::<Result<Box<[u8]>, LuaError>>()?;

                Ok(Self::new(bytes))
            }

            _ => Err(LuaError::external("can't convert value into Bytes type"))
        }
    }
}

impl LuaUserData for Bytes {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("len", |_: &Lua, bytes: &Self| Ok(bytes.len));
        fields.add_field_method_get("pos", |_: &Lua, bytes: &Self| Ok(bytes.pos));
        fields.add_field_method_get("is_empty", |_: &Lua, bytes: &Self| Ok(bytes.is_empty()));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method("__len", |_: &Lua, bytes: &Self, _: ()| Ok(bytes.len));
        methods.add_meta_method("__eq", |_: &Lua, bytes: &Self, other: Bytes| Ok(bytes.as_slice() == other.as_slice()));
        methods.add_meta_method("__lt", |_: &Lua, bytes: &Self, other: Bytes| Ok(bytes.as_slice() < other.as_slice()));
        methods.add_meta_method("__le", |_: &Lua, bytes: &Self, other: Bytes| Ok(bytes.as_slice() <= other.as_slice()));

        methods.add_meta_method("__index", |_: &Lua, bytes: &Self, idx: usize| {
            if idx > 0 && let Some(byte) = bytes.as_slice().get(idx - 1) {
                Ok(LuaValue::Integer(*byte as i64))
            } else  {
                Ok(LuaValue::Nil)
            }
        });

        methods.add_method("as_table", |lua: &Lua, bytes: &Self, _: ()| {
            lua.create_sequence_from(bytes.iter().copied())
        });

        methods.add_method("as_string", |lua: &Lua, bytes: &Self, _: ()| {
            lua.create_string(&bytes.buf)
        });

        methods.add_method_mut("read", |lua: &Lua, bytes: &mut Self, _: ()| {
            let mut buf = Vec::new();

            let n = bytes.read(&mut buf)?;

            if n == 0 {
                return Ok(LuaValue::Nil);
            }

            let table = lua.create_table_with_capacity(n, 0)?;

            for byte in buf {
                table.raw_push(byte)?;
            }

            Ok(LuaValue::Table(table))
        });

        methods.add_method_mut("read_exact", |lua: &Lua, bytes: &mut Self, len: usize| {
            let mut buf = Vec::with_capacity(len);

            bytes.read_exact(&mut buf)?;

            let table = lua.create_table_with_capacity(len, 0)?;

            for byte in buf {
                table.raw_push(byte)?;
            }

            Ok(LuaValue::Table(table))
        });

        methods.add_method_mut("seek", |_: &Lua, bytes: &mut Self, pos: i64| {
            if pos >= 0 {
                bytes.seek(SeekFrom::Start(pos as u64))?;
            } else {
                bytes.seek(SeekFrom::End(pos))?;
            }

            Ok(LuaValue::Integer(bytes.pos as i64))
        });

        methods.add_method_mut("seek_rel", |_: &Lua, bytes: &mut Self, offset: i64| {
            bytes.seek(SeekFrom::Current(offset))?;

            Ok(LuaValue::Integer(bytes.pos as i64))
        });
    }
}

impl AsRef<Bytes> for Bytes {
    #[inline(always)]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl std::ops::Deref for Bytes {
    type Target = Box<[u8]>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl From<Box<[u8]>> for Bytes {
    #[inline(always)]
    fn from(value: Box<[u8]>) -> Self {
        Self::new(value)
    }
}

impl From<Vec<u8>> for Bytes {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self::new(value.into_boxed_slice())
    }
}

impl From<Bytes> for Box<[u8]> {
    #[inline(always)]
    fn from(value: Bytes) -> Self {
        value.buf
    }
}

impl From<Bytes> for Vec<u8> {
    #[inline]
    fn from(value: Bytes) -> Self {
        value.buf.to_vec()
    }
}
