use mlua::prelude::*;

use encoding_rs::Encoding;

use super::*;

#[allow(clippy::large_enum_variant)]
enum StringEncoding {
    Base16,
    Base32(base32::Alphabet),
    Base64(base64::engine::GeneralPurpose),
    Json,
    Toml
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
            b"toml" => Some(Self::Toml),

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

            Self::Toml => {
                let value = toml::to_string(&value)
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

            Self::Toml => {
                let string = string.to_string_lossy()
                    .to_string();

                let value = toml::from_str::<toml::Value>(&string)
                    .map_err(LuaError::external)?;

                lua.to_value(&value)
            }
        }
    }
}

pub struct StringAPI<'lua> {
    lua: &'lua Lua,

    str_to_bytes: LuaFunction<'lua>,
    str_from_bytes: LuaFunction<'lua>,
    str_encode: LuaFunction<'lua>,
    str_decode: LuaFunction<'lua>
}

impl<'lua> StringAPI<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, PackagesEngineError> {
        Ok(Self {
            lua,

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
            })?
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable<'lua>, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 4)?;

        env.set("to_bytes", self.str_to_bytes.clone())?;
        env.set("from_bytes", self.str_from_bytes.clone())?;
        env.set("encode", self.str_encode.clone())?;
        env.set("decode", self.str_decode.clone())?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_bytes() -> anyhow::Result<()> {
        let lua = Lua::new();
        let api = StringAPI::new(&lua)?;

        assert_eq!(api.str_to_bytes.call::<_, Vec<u8>>("abc")?, &[97, 98, 99]);
        assert_eq!(api.str_to_bytes.call::<_, Vec<u8>>(0.5)?, &[63, 224, 0, 0, 0, 0, 0, 0]);
        assert_eq!(api.str_to_bytes.call::<_, Vec<u8>>(vec![1, 2, 3])?, &[1, 2, 3]);

        assert_eq!(api.str_to_bytes.call::<_, Vec<u8>>("абоба")?, &[208, 176, 208, 177, 208, 190, 208, 177, 208, 176]);
        assert_eq!(api.str_to_bytes.call::<_, Vec<u8>>(("абоба", "cp1251"))?, &[224, 225, 238, 225, 224]);

        assert_eq!(api.str_from_bytes.call::<_, LuaString>(vec![97, 98, 99])?, b"abc");

        assert_eq!(api.str_from_bytes.call::<_, LuaString>(vec![208, 176, 208, 177, 208, 190, 208, 177, 208, 176])?, "абоба");
        assert_eq!(api.str_from_bytes.call::<_, LuaString>((vec![224, 225, 238, 225, 224], "cp1251"))?, "абоба");

        Ok(())
    }

    #[test]
    fn str_encodings() -> anyhow::Result<()> {
        let lua = Lua::new();
        let api = StringAPI::new(&lua)?;

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
            let encoded = api.str_encode.call::<_, LuaString>(("Hello, World!", name))?;
            let decoded = api.str_decode.call::<_, Vec<u8>>((value, name))?;

            assert_eq!(encoded, value);
            assert_eq!(decoded, b"Hello, World!");
        }

        let table = lua.create_table()?;

        table.set("hello", "world")?;

        let encodings = [
            ("json", "{\"hello\":\"world\"}"),
            ("toml", "hello = \"world\"\n")
        ];

        for (name, value) in encodings {
            let encoded = api.str_encode.call::<_, LuaString>((table.clone(), name))?;
            let decoded = api.str_decode.call::<_, LuaTable>((value, name))?;

            assert_eq!(encoded, value);
            assert_eq!(decoded.get::<_, LuaString>("hello")?, "world");
        }

        Ok(())
    }
}
