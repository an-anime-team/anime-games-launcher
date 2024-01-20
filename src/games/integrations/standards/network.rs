use std::collections::HashMap;

use mlua::prelude::*;

pub use anime_game_core::network::minreq::Method as RequestMethod;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestOptions {
    /// Request method
    pub method: Option<RequestMethod>,

    /// Request headers
    pub headers: Option<HashMap<String, String>>,

    /// Request body
    pub body: Option<Vec<u8>>,

    /// Request timeout, in seconds
    pub timeout: Option<u64>
}

impl<'lua> FromLua<'lua> for RequestOptions {
    fn from_lua(value: LuaValue<'lua>, _lua: &'lua Lua) -> LuaResult<Self> {
        let Some(table) = value.as_table() else {
            return Err(LuaError::UserDataTypeMismatch);
        };

        Ok(Self {
            method: table.contains_key("method")?
                .then(|| {
                    let method = table.get::<_, String>("method")?;

                    let method = match method.as_str() {
                        "get"     => RequestMethod::Get,
                        "head"    => RequestMethod::Head,
                        "post"    => RequestMethod::Post,
                        "put"     => RequestMethod::Put,
                        "delete"  => RequestMethod::Delete,
                        "connect" => RequestMethod::Connect,
                        "options" => RequestMethod::Options,
                        "trace"   => RequestMethod::Trace,
                        "patch"   => RequestMethod::Patch,

                        _ => RequestMethod::Custom(method)
                    };

                    Ok::<_, LuaError>(method)
                })
                .transpose()?,

            headers: table.contains_key("headers")?
                .then(|| {
                    let headers = table.get::<_, LuaTable>("headers")?
                        .pairs::<String, String>()
                        .flatten()
                        .collect::<HashMap<_, _>>();

                    Ok::<_, LuaError>(headers)
                })
                .transpose()?,

            body: table.contains_key("body")?
                .then(|| {
                    if let Ok(string) = table.get::<_, LuaString>("body") {
                        Ok(string.as_bytes().to_vec())
                    }

                    else if let Ok(table) = table.get::<_, LuaTable>("body") {
                        Ok(table.sequence_values::<u8>().flatten().collect())
                    }

                    else {
                        Err(LuaError::UserDataTypeMismatch)
                    }
                })
                .transpose()?,

            timeout: table.contains_key("timeout")?
                .then(|| table.get("timeout"))
                .transpose()?
        })
    }
}
