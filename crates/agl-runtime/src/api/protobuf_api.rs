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

use std::collections::HashMap;

use mlua::prelude::*;

use protox::file::{File, FileResolver};
use prost_reflect::{
    DescriptorPool, DynamicMessage, MapKey, ReflectMessage,
    Value as ProtobufValue, Kind
};
use prost_reflect::prost_types::FileDescriptorSet;
use protox::prost::Message;

use super::bytes::Bytes;

/// Create `DescriptorPool` from a given `.proto` file schema.
fn create_pool(schema: String) -> Result<DescriptorPool, protox::Error> {
    struct StringResolver(String);

    impl FileResolver for StringResolver {
        fn open_file(&self, name: &str) -> Result<File, protox::Error> {
            if name == "schema.proto" {
                File::from_source(name, &self.0)
            } else {
                Err(protox::Error::file_not_found(name))
            }
        }
    }

    let fds: FileDescriptorSet = protox::Compiler::with_file_resolver(StringResolver(schema))
        .include_imports(true)
        .open_file("schema.proto")?
        .file_descriptor_set();

    DescriptorPool::from_file_descriptor_set(fds)
        .map_err(|err| protox::Error::new(err.to_string()))
}

fn lua_to_protobuf_value(
    value: LuaValue,
    field_kind: Option<&Kind>
) -> Result<ProtobufValue, LuaError> {
    match value {
        LuaValue::String(value) => Ok(ProtobufValue::String(value.to_string_lossy().to_string())),

        LuaValue::Number(value) => {
            match field_kind {
                Some(Kind::Int32)    => Ok(ProtobufValue::I32(value as i32)),
                Some(Kind::Int64)    => Ok(ProtobufValue::I64(value as i64)),
                Some(Kind::Uint32)   => Ok(ProtobufValue::U32(value as u32)),
                Some(Kind::Uint64)   => Ok(ProtobufValue::U64(value as u64)),
                Some(Kind::Sint32)   => Ok(ProtobufValue::I32(value as i32)),
                Some(Kind::Sint64)   => Ok(ProtobufValue::I64(value as i64)),
                Some(Kind::Fixed32)  => Ok(ProtobufValue::I32(value as i32)),
                Some(Kind::Sfixed32) => Ok(ProtobufValue::I32(value as i32)),
                Some(Kind::Fixed64)  => Ok(ProtobufValue::I64(value as i64)),
                Some(Kind::Sfixed64) => Ok(ProtobufValue::I64(value as i64)),
                Some(Kind::Float)    => Ok(ProtobufValue::F32(value as f32)),
                Some(Kind::Double)   => Ok(ProtobufValue::F64(value)),

                _ => {
                    if value.is_finite() && (value as f32 as f64) == value {
                        Ok(ProtobufValue::F32(value as f32))
                    } else {
                        Ok(ProtobufValue::F64(value))
                    }
                }
            }
        }

        LuaValue::Integer(value) => {
            match field_kind {
                Some(Kind::Int32)    => Ok(ProtobufValue::I32(value as i32)),
                Some(Kind::Int64)    => Ok(ProtobufValue::I64(value)),
                Some(Kind::Uint32)   => Ok(ProtobufValue::U32(value as u32)),
                Some(Kind::Uint64)   => Ok(ProtobufValue::U64(value as u64)),
                Some(Kind::Sint32)   => Ok(ProtobufValue::I32(value as i32)),
                Some(Kind::Sint64)   => Ok(ProtobufValue::I64(value)),
                Some(Kind::Fixed32)  => Ok(ProtobufValue::I32(value as i32)),
                Some(Kind::Sfixed32) => Ok(ProtobufValue::I32(value as i32)),
                Some(Kind::Fixed64)  => Ok(ProtobufValue::I64(value)),
                Some(Kind::Sfixed64) => Ok(ProtobufValue::I64(value)),
                Some(Kind::Float)    => Ok(ProtobufValue::F32(value as f32)),
                Some(Kind::Double)   => Ok(ProtobufValue::F64(value as f64)),

                _ => {
                    if value.abs() <= i32::MAX as i64 {
                        Ok(ProtobufValue::I32(value as i32))
                    } else {
                        Ok(ProtobufValue::I64(value))
                    }
                }
            }
        }

        LuaValue::Boolean(value) => Ok(ProtobufValue::Bool(value)),

        LuaValue::Table(values) => {
            let total_len = values.raw_len();
            let sequence_len = values.sequence_values::<LuaValue>().count();

            if sequence_len == total_len {
                let mut list = Vec::with_capacity(sequence_len);

                for value in values.sequence_values::<LuaValue>() {
                    list.push(lua_to_protobuf_value(value?, field_kind)?);
                }

                Ok(ProtobufValue::List(list))
            }

            else {
                let mut map = HashMap::with_capacity(values.raw_len());

                for pair in values.pairs::<LuaValue, LuaValue>() {
                    let (key, value) = pair?;

                    let key = match key {
                        LuaValue::String(value)  => MapKey::String(value.to_string_lossy().to_string()),
                        LuaValue::Number(value)  => MapKey::I64(value as i64),
                        LuaValue::Integer(value) => MapKey::I64(value),
                        LuaValue::Boolean(value) => MapKey::Bool(value),

                        _ => return Err(LuaError::external("invalid protobuf map key")
                            .context(format!("{key:#?}")))
                    };

                    map.insert(key, lua_to_protobuf_value(value, field_kind)?);
                }

                Ok(ProtobufValue::Map(map))
            }
        }

        LuaValue::Function(function) => lua_to_protobuf_value(function.call::<LuaValue>(())?, field_kind),

        LuaValue::UserData(ref object) if object.type_name()?.as_deref() == Some("Bytes") => {
            let bytes = object.call_method::<Vec<u8>>("as_table", ())?;

            Ok(ProtobufValue::Bytes(bytes.into()))
        }

        _ => Err(LuaError::external("can't coerce lua value to a protobuf value")
            .context(format!("{value:#?}")))
    }
}

fn protobuf_to_lua_value(
    lua: &Lua,
    value: &ProtobufValue
) -> Result<LuaValue, LuaError> {
    match value {
        ProtobufValue::String(value) => Ok(LuaValue::String(lua.create_string(value)?)),
        ProtobufValue::I32(value)    => Ok(LuaValue::Integer(*value as i64)),
        ProtobufValue::I64(value)    => Ok(LuaValue::Integer(*value)),
        ProtobufValue::U32(value)    => Ok(LuaValue::Integer(*value as i64)),
        ProtobufValue::U64(value)    => Ok(LuaValue::Integer(*value as i64)),
        ProtobufValue::F32(value)    => Ok(LuaValue::Number(*value as f64)),
        ProtobufValue::F64(value)    => Ok(LuaValue::Number(*value)),
        ProtobufValue::Bool(value)   => Ok(LuaValue::Boolean(*value)),

        ProtobufValue::List(values) => {
            let list = lua.create_table_with_capacity(values.len(), 0)?;

            for value in values {
                list.raw_push(protobuf_to_lua_value(lua, value)?)?;
            }

            Ok(LuaValue::Table(list))
        }

        ProtobufValue::Map(values) => {
            let list = lua.create_table_with_capacity(0, values.len())?;

            for (key, value) in values {
                let key = match key {
                    MapKey::String(value) => LuaValue::String(lua.create_string(value)?),
                    MapKey::I32(value)    => LuaValue::Integer(*value as i64),
                    MapKey::I64(value)    => LuaValue::Integer(*value),
                    MapKey::U32(value)    => LuaValue::Integer(*value as i64),
                    MapKey::U64(value)    => LuaValue::Integer(*value as i64),
                    MapKey::Bool(value)   => LuaValue::Boolean(*value)
                };

                list.raw_set(key, protobuf_to_lua_value(lua, value)?)?;
            }

            Ok(LuaValue::Table(list))
        }

        ProtobufValue::Bytes(bytes) => {
            let bytes = Bytes::new(bytes.to_vec().into_boxed_slice());

            Ok(LuaValue::UserData(lua.create_userdata(bytes)?))
        }

        _ => Err(LuaError::external("can't coerce protobuf value to a lua value")
            .context(value))
    }
}

/// Encode given protobuf `DynamicMessage` with values stored in the given
/// lua table.
fn protobuf_encode(
    mut message: DynamicMessage,
    values: LuaTable
) -> Result<Bytes, LuaError> {
    for pair in values.pairs::<LuaValue, LuaValue>() {
        let (name, value) = pair?;

        if !value.is_nil() && !value.is_null() {
            match name {
                LuaValue::Integer(index) => {
                    let value = match message.descriptor().get_field(index as u32 + 1) {
                        Some(field) => lua_to_protobuf_value(value, Some(&field.kind()))?,
                        None =>  lua_to_protobuf_value(value, None)?
                    };

                    message.try_set_field_by_number(index as u32 + 1, value)
                        .map_err(LuaError::external)?;
                }

                LuaValue::String(name) => {
                    let value = match message.descriptor().get_field_by_name(&name.to_string_lossy()) {
                        Some(field) => lua_to_protobuf_value(value, Some(&field.kind()))?,
                        None => lua_to_protobuf_value(value, None)?
                    };

                    message.try_set_field_by_name(&name.to_string_lossy(), value)
                        .map_err(LuaError::external)?;
                }

                _ => return Err(LuaError::external("invalid protobuf values index type")
                    .context(format!("{name:#?}")))
            }
        }
    }

    Ok(Bytes::new(message.encode_to_vec().into_boxed_slice()))
}

/// Decode given protobuf `DynamicMessage` into a lua table values.
fn protobuf_decode(
    lua: &Lua,
    message: DynamicMessage
) -> Result<LuaTable, LuaError> {
    let values = lua.create_table()?;

    for (field, value) in message.fields() {
        let value = protobuf_to_lua_value(lua, value)?;

        values.raw_set(field.name(), &value)?;
        values.raw_push(value)?;
    }

    Ok(values)
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Protobuf(DescriptorPool);

impl FromLua for Protobuf {
    fn from_lua(value: LuaValue, _lua: &Lua) -> LuaResult<Self> {
        match value {
            LuaValue::String(str) => {
                let mut protobuf = Protobuf::default();

                protobuf.decode_file_descriptor_set(str.to_string_lossy().as_bytes())
                    .map_err(LuaError::external)?;

                Ok(protobuf)
            }

            LuaValue::Table(table) => {
                let bytes = table.sequence_values::<u8>()
                    .collect::<Result<Vec<u8>, LuaError>>()?;

                let mut protobuf = Protobuf::default();

                protobuf.decode_file_descriptor_set(bytes.as_slice())
                    .map_err(LuaError::external)?;

                Ok(protobuf)
            }

            LuaValue::UserData(object) if object.type_name()?.as_deref() == Some("Bytes") => {
                let table = object.call_method::<LuaTable>("as_table", ())?;

                let bytes = table.sequence_values::<u8>()
                    .collect::<Result<Vec<u8>, LuaError>>()?;

                let mut protobuf = Protobuf::default();

                protobuf.decode_file_descriptor_set(bytes.as_slice())
                    .map_err(LuaError::external)?;

                Ok(protobuf)
            }

            LuaValue::UserData(object) if object.type_name()?.as_deref() == Some("Protobuf") => {
                let bytes = object.call_method::<Bytes>("as_bytes", ())?;

                let mut protobuf = Protobuf::default();

                protobuf.decode_file_descriptor_set(bytes.as_slice())
                    .map_err(LuaError::external)?;

                Ok(protobuf)
            }

            _ => Err(LuaError::external("can't convert value into Bytes type"))
        }
    }
}

impl LuaUserData for Protobuf {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("as_bytes", |_lua: &Lua, protobuf: &Self, _: ()| {
            Ok(Bytes::new(protobuf.encode_to_vec().into_boxed_slice()))
        });

        methods.add_function("from_bytes", |_lua: &Lua, schema: Bytes| {
            let mut protobuf = Protobuf::default();

            protobuf.decode_file_descriptor_set(schema.as_slice())
                .map_err(LuaError::external)?;

            Ok(protobuf)
        });
    }
}

impl From<DescriptorPool> for Protobuf {
    #[inline(always)]
    fn from(value: DescriptorPool) -> Self {
        Self(value)
    }
}

impl From<Protobuf> for DescriptorPool {
    #[inline(always)]
    fn from(value: Protobuf) -> Self {
        value.0
    }
}

impl AsRef<Protobuf> for Protobuf {
    #[inline(always)]
    fn as_ref(&self) -> &Protobuf {
        self
    }
}

impl std::ops::Deref for Protobuf {
    type Target = DescriptorPool;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Protobuf {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct ProtobufApi {
    lua: Lua,

    protobuf_create: LuaFunction,
    protobuf_encode: LuaFunction,
    protobuf_decode: LuaFunction
}

impl ProtobufApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        Ok(Self {
            protobuf_create: lua.create_function(move |lua: &Lua, schema: String| {
                let pool = create_pool(schema)
                    .map_err(LuaError::external)?;

                Protobuf::from(pool)
                    .into_lua(lua)
            })?,

            protobuf_encode: lua.create_function(move |_lua: &Lua, (schema, message, values): (Protobuf, String, LuaTable)| {
                let Some(message) = schema.get_message_by_name(&message) else {
                    return Err(LuaError::external("no such message"));
                };

                let message = DynamicMessage::new(message);

                protobuf_encode(message, values)
            })?,

            protobuf_decode: lua.create_function(move |lua: &Lua, (schema, message, value): (Protobuf, String, Bytes)| {
                let Some(message) = schema.get_message_by_name(&message) else {
                    return Err(LuaError::external("no such message"));
                };

                let message = DynamicMessage::decode(message, value.as_slice())
                    .map_err(LuaError::external)?;

                protobuf_decode(lua, message)
            })?,

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 3)?;

        env.raw_set("create", &self.protobuf_create)?;
        env.raw_set("encode", &self.protobuf_encode)?;
        env.raw_set("decode", &self.protobuf_decode)?;

        Ok(env)
    }
}
