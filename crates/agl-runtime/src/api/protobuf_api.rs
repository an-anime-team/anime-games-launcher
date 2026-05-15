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

use mlua::prelude::*;

pub struct ProtobufApi {
    lua: Lua,

    protobuf_create: LuaFunction,
    protobuf_encode: LuaFunction,
    protobuf_decode: LuaFunction
}

impl ProtobufApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        Ok(Self {
            protobuf_create: lua.create_function(move |_lua: &Lua, _: ()| {
                Ok(())
            })?,

            protobuf_encode: lua.create_function(move |_lua: &Lua, _: ()| {
                Ok(())
            })?,

            protobuf_decode: lua.create_function(move |_lua: &Lua, _: ()| {
                Ok(())
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
