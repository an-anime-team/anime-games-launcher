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

use mlua::prelude::*;

use encoding_rs::Encoding;

use super::bytes::Bytes;

/// Filter provided lua value to keep only basic types like numbers, booleans,
/// strings and tables.
fn filter_lua_value(value: LuaValue) -> Result<LuaValue, LuaError> {
    match value {
        LuaValue::Integer(_) |
        LuaValue::Number(_) |
        LuaValue::Boolean(_) |
        LuaValue::String(_) => Ok(value),

        LuaValue::Table(table) => {
            table.for_each::<LuaValue, LuaValue>(|key, value| {
                table.raw_set(key, filter_lua_value(value)?)?;

                Ok(())
            })?;

            Ok(LuaValue::Table(table))
        }

        // Bytes userdata
        LuaValue::UserData(object) if object.type_name()?.as_deref() == Some("Bytes") => {
            object.call_method::<LuaTable>("as_table", ())
                .map(LuaValue::Table)
        }

        _ => Ok(LuaValue::Nil)
    }
}

#[allow(clippy::large_enum_variant)]
enum StringEncoding {
    Base16,
    Base32(base32::Alphabet),
    Base64(base64::engine::GeneralPurpose),
    Json { pretty: bool },
    Bson,
    Toml { pretty: bool },
    Yaml
}

impl StringEncoding {
    pub fn encode(&self, lua: &Lua, value: LuaValue) -> Result<LuaString, LuaError> {
        use base64::Engine;

        match self {
            Self::Base16 => {
                let value = Bytes::from_lua(value, lua)?;

                lua.create_string(hex::encode(value.as_slice()))
            }

            Self::Base32(alphabet) => {
                let value = Bytes::from_lua(value, lua)?;

                lua.create_string(base32::encode(*alphabet, value.as_slice()))
            }

            Self::Base64(engine) => {
                let value = Bytes::from_lua(value, lua)?;

                lua.create_string(engine.encode(value.as_slice()))
            }

            Self::Json { pretty } => {
                let value = if *pretty {
                    serde_json::to_vec_pretty(&value)
                } else {
                    serde_json::to_vec(&value)
                };

                lua.create_string(value.map_err(LuaError::external)?)
            }

            Self::Bson => {
                let value = bson::serialize_to_vec(&value)
                    .map_err(LuaError::external)?;

                lua.create_string(value)
            }

            Self::Toml { pretty } => {
                let value = if *pretty {
                    toml::to_string_pretty(&value)
                } else {
                    toml::to_string(&value)
                };

                lua.create_string(value.map_err(LuaError::external)?)
            }

            Self::Yaml => {
                let value = serde_yml::to_string(&value)
                    .map_err(LuaError::external)?;

                lua.create_string(value)
            }
        }
    }

    pub fn decode(&self, lua: &Lua, string: LuaString) -> Result<LuaValue, LuaError> {
        use base64::Engine;

        match self {
            Self::Base16 => {
                let value = hex::decode(string.as_bytes())
                    .map_err(LuaError::external)?;

                Bytes::new(value.into_boxed_slice())
                    .into_lua(lua)
            }

            Self::Base32(alphabet) => {
                let string = string.to_string_lossy()
                    .to_string();

                let value = base32::decode(*alphabet, &string)
                    .ok_or_else(|| LuaError::external("invalid base32 string"))?;

                Bytes::new(value.into_boxed_slice())
                    .into_lua(lua)
            }

            Self::Base64(engine) => {
                let value = engine.decode(string.as_bytes())
                    .map_err(LuaError::external)?;

                Bytes::new(value.into_boxed_slice())
                    .into_lua(lua)
            }

            Self::Json { .. } => {
                let value = serde_json::from_slice::<serde_json::Value>(&string.as_bytes())
                    .map_err(LuaError::external)?;

                Ok(filter_lua_value(lua.to_value(&value)?)?)
            }

            Self::Bson => {
                let value = bson::deserialize_from_slice::<bson::Bson>(&string.as_bytes())
                    .map_err(LuaError::external)?;

                Ok(filter_lua_value(lua.to_value(&value)?)?)
            }

            Self::Toml { .. } => {
                let string = string.to_string_lossy()
                    .to_string();

                let value = toml::from_str::<toml::Value>(&string)
                    .map_err(LuaError::external)?;

                Ok(filter_lua_value(lua.to_value(&value)?)?)
            }

            Self::Yaml => {
                let value = serde_yml::from_slice::<serde_yml::Value>(&string.as_bytes())
                    .map_err(LuaError::external)?;

                Ok(filter_lua_value(lua.to_value(&value)?)?)
            }
        }
    }
}

impl FromStr for StringEncoding {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "base16" | "hex" => Ok(Self::Base16),

            // Base32
            "base32" | "base32/pad" => {
                Ok(Self::Base32(base32::Alphabet::Rfc4648Lower { padding: true }))
            }

            "base32/nopad" => {
                Ok(Self::Base32(base32::Alphabet::Rfc4648Lower { padding: false }))
            }

            "base32/hex-pad"   => {
                Ok(Self::Base32(base32::Alphabet::Rfc4648HexLower { padding: true }))
            }

            "base32/hex-nopad" => {
                Ok(Self::Base32(base32::Alphabet::Rfc4648HexLower { padding: false }))
            }

            // Base64
            "base64" | "base64/pad" => {
                let encoding = base64::engine::GeneralPurpose::new(
                    &base64::alphabet::STANDARD,
                    base64::engine::GeneralPurposeConfig::new()
                        .with_encode_padding(true)
                );

                Ok(Self::Base64(encoding))
            }

            "base64/nopad" => {
                let encoding = base64::engine::GeneralPurpose::new(
                    &base64::alphabet::STANDARD,
                    base64::engine::GeneralPurposeConfig::new()
                        .with_encode_padding(false)
                );

                Ok(Self::Base64(encoding))
            }

            "base64/urlsafe-pad" => {
                let encoding = base64::engine::GeneralPurpose::new(
                    &base64::alphabet::URL_SAFE,
                    base64::engine::GeneralPurposeConfig::new()
                        .with_encode_padding(true)
                );

                Ok(Self::Base64(encoding))
            }

            "base64/urlsafe-nopad" => {
                let encoding = base64::engine::GeneralPurpose::new(
                    &base64::alphabet::URL_SAFE,
                    base64::engine::GeneralPurposeConfig::new()
                        .with_encode_padding(false)
                );

                Ok(Self::Base64(encoding))
            }

            "json" | "json/compact"  => Ok(Self::Json { pretty: false }),
            "json/pretty" => Ok(Self::Json { pretty: true }),

            "bson" => Ok(Self::Bson),

            "toml" | "toml/compact" => Ok(Self::Toml { pretty: false }),
            "toml/pretty" => Ok(Self::Toml { pretty: true }),

            "yaml" => Ok(Self::Yaml),

            _ => Err(())
        }
    }
}

#[derive(Debug)]
pub struct StringApi {
    lua: Lua,

    str_to_bytes: LuaFunction,
    str_from_bytes: LuaFunction,
    str_encode: LuaFunction,
    str_decode: LuaFunction
}

impl StringApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        Ok(Self {
            str_to_bytes: lua.create_function(|lua: &Lua, (value, charset): (Bytes, Option<LuaString>)| {
                let Some(charset) = charset else {
                    return value.into_lua(lua);
                };

                let Some(charset) = Encoding::for_label(&charset.as_bytes()) else {
                    return Err(LuaError::external("invalid charset"));
                };

                let value = String::from_utf8(value.to_vec())
                    .map_err(|err| LuaError::external(format!("utf-8 string expected: {err}")))?;

                let value = charset.encode(&value).0.to_vec();

                Bytes::new(value.into_boxed_slice())
                    .into_lua(lua)
            })?,

            str_from_bytes: lua.create_function(|lua, (value, charset): (Bytes, Option<LuaString>)| {
                let Some(charset) = charset else {
                    return lua.create_string(value.as_slice());
                };

                let Some(charset) = Encoding::for_label(&charset.as_bytes()) else {
                    return Err(LuaError::external("invalid charset"));
                };

                let value = charset.decode(&value).0;

                lua.create_string(value.as_bytes())
            })?,

            str_encode: lua.create_function(|lua, (encoding, value): (String, LuaValue)| {
                let Ok(encoding) = StringEncoding::from_str(&encoding) else {
                    return Err(LuaError::external("invalid encoding"));
                };

                encoding.encode(lua, value)
            })?,

            str_decode: lua.create_function(|lua, (encoding, value): (String, LuaString)| {
                let Ok(encoding) = StringEncoding::from_str(&encoding) else {
                    return Err(LuaError::external("invalid encoding"));
                };

                encoding.decode(lua, value)
            })?,

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 4)?;

        env.raw_set("to_bytes", &self.str_to_bytes)?;
        env.raw_set("from_bytes", &self.str_from_bytes)?;
        env.raw_set("encode", &self.str_encode)?;
        env.raw_set("decode", &self.str_decode)?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_encodings() -> Result<(), LuaError> {
        let api = StringApi::new(Lua::new())?;

        assert_eq!(api.str_to_bytes.call::<Bytes>("abc")?.as_slice(), &[97, 98, 99]);
        assert_eq!(api.str_to_bytes.call::<Bytes>([1, 2, 3])?.as_slice(), &[1, 2, 3]);

        assert_eq!(api.str_to_bytes.call::<Bytes>("абоба")?.as_slice(), &[208, 176, 208, 177, 208, 190, 208, 177, 208, 176]);
        assert_eq!(api.str_to_bytes.call::<Bytes>(("абоба", "cp1251"))?.as_slice(), &[224, 225, 238, 225, 224]);

        assert_eq!(api.str_from_bytes.call::<LuaString>(vec![97, 98, 99])?, b"abc");

        assert_eq!(api.str_from_bytes.call::<LuaString>(vec![208, 176, 208, 177, 208, 190, 208, 177, 208, 176])?, "абоба");
        assert_eq!(api.str_from_bytes.call::<LuaString>((vec![224, 225, 238, 225, 224], "cp1251"))?, "абоба");

        Ok(())
    }

    #[test]
    fn text_formats() -> Result<(), LuaError> {
        let lua = Lua::new();
        let api = StringApi::new(lua.clone())?;

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
            let encoded = api.str_encode.call::<LuaString>((name, "Hello, World!"))?;
            let decoded = api.str_decode.call::<Bytes>((name, value))?;

            assert_eq!(encoded, value);
            assert_eq!(decoded.as_slice(), b"Hello, World!");
        }

        let table = lua.create_table_with_capacity(0, 3)?;

        table.set("test_string", "str")?;
        table.set("test_null", LuaValue::Nil)?;
        table.set("test_bool", true)?;

        let encodings = [
            ("json", b"{ \"test_string\": \"str\", \"test_bool\": true, \"test_null\": null }".as_slice()),
            ("bson", [0x31, 0x00, 0x00, 0x00, 0x02, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x00, 0x04, 0x00, 0x00, 0x00, 0x73, 0x74, 0x72, 0x00, 0x0a, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x6e, 0x75, 0x6c, 0x6c, 0x00, 0x08, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x62, 0x6f, 0x6f, 0x6c, 0x00, 0x01, 0x00].as_slice()),
            ("toml", b"test_string = \"str\"\ntest_bool = true".as_slice()),
            ("yaml", b"test_string: \"str\"\ntest_bool: true\ntest_null: null".as_slice())
        ];

        for (name, value) in encodings {
            let value = lua.create_string(value)?;

            let encoded = api.str_encode.call::<LuaString>((name, table.clone()))?;

            let decoded_1 = api.str_decode.call::<LuaTable>((name, value))?;
            let decoded_2 = api.str_decode.call::<LuaTable>((name, encoded))?;

            assert_eq!(decoded_1.get::<LuaString>("test_string")?, "str");
            assert_eq!(decoded_1.get::<LuaValue>("test_bool")?, LuaValue::Boolean(true));

            assert_eq!(decoded_2.get::<LuaString>("test_string")?, "str");
            assert_eq!(decoded_2.get::<LuaValue>("test_bool")?, LuaValue::Boolean(true));

            if name != "toml" {
                assert_eq!(decoded_1.get::<LuaValue>("test_null")?, LuaValue::Nil);
                assert_eq!(decoded_2.get::<LuaValue>("test_null")?, LuaValue::Nil);
            }
        }

        Ok(())
    }
}
