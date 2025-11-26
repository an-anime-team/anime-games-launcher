use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use wineyard_core::export::network::reqwest::{Client, RequestBuilder, Method};
use wineyard_core::tasks;

use mlua::prelude::*;

use super::*;

fn create_request(
    client: &Client,
    url: impl AsRef<str>,
    options: Option<LuaTable>
) -> Result<RequestBuilder, LuaError> {
    let mut method = String::from("get");

    // Change the request method if provided.
    if let Some(options) = &options {
        method = options.get::<String>("method")
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
        if let Ok(headers) = options.get::<LuaTable>("headers") {
            for pair in headers.pairs::<LuaString, LuaString>() {
                let (key, value) = pair?;

                request = request.header(
                    key.to_string_lossy().to_string(),
                    value.to_string_lossy().to_string()
                );
            }
        }

        if let Ok(body) = options.get::<LuaValue>("body") {
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

pub struct NetworkAPI {
    lua: Lua,

    net_fetch: LuaFunction,
    net_open: LuaFunction,
    net_read: LuaFunction,
    net_close: LuaFunction
}

impl NetworkAPI {
    pub fn new(lua: Lua, client: Client) -> Result<Self, PackagesEngineError> {
        let net_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            net_fetch: {
                let client = client.clone();

                lua.create_function(move |lua, (url, options): (LuaString, Option<LuaTable>)| {
                    let url = url.to_string_lossy().to_string();
                    let request = create_request(&client, url, options)?;

                    // Perform the request.
                    let response = tasks::block_on(async move {
                        let result = lua.create_table_with_capacity(0, 4)?;
                        let headers = lua.create_table()?;

                        let response = request.send().await
                            .map_err(|err| LuaError::external(format!("failed to perform request: {err}")))?;

                        result.raw_set("status", response.status().as_u16())?;
                        result.raw_set("is_ok", response.status().is_success())?;
                        result.raw_set("headers", headers.clone())?;

                        for (key, value) in response.headers() {
                            headers.raw_set(key.to_string(), lua.create_string(value.as_bytes())?)?;
                        }

                        let body = response.bytes().await
                            .map_err(|err| LuaError::external(format!("failed to fetch body: {err}")))?;

                        result.raw_set("body", body.to_vec())?;

                        Ok::<_, LuaError>(result)
                    })?;

                    Ok(response)
                })?
            },

            net_open: {
                let client = client.clone();
                let net_handles = net_handles.clone();

                lua.create_function(move |lua, (url, options): (LuaString, Option<LuaTable>)| {
                    let url = url.to_string_lossy().to_string();
                    let request = create_request(&client, url, options)?;

                    let (response, header) = tasks::block_on(async move {
                        let result = lua.create_table_with_capacity(0, 3)?;
                        let headers = lua.create_table()?;

                        let response = request.send().await
                            .map_err(|err| LuaError::external(format!("failed to perform request: {err}")))?;

                        result.raw_set("status", response.status().as_u16())?;
                        result.raw_set("is_ok", response.status().is_success())?;
                        result.raw_set("headers", headers.clone())?;

                        for (key, value) in response.headers() {
                            headers.raw_set(key.to_string(), lua.create_string(value.as_bytes())?)?;
                        }

                        Ok::<_, LuaError>((response, result))
                    })?;

                    let mut handles = net_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                    let mut handle = rand::random::<i32>();

                    while handles.contains_key(&handle) {
                        handle = rand::random::<i32>();
                    }

                    handles.insert(handle, response);

                    header.raw_set("handle", handle)?;

                    Ok(header)
                })?
            },

            net_read: {
                let net_handles = net_handles.clone();

                lua.create_function(move |lua, handle: i32| {
                    let mut handles = net_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(response) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid request handle"));
                    };

                    let chunk = tasks::block_on(async move {
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

                lua.create_function(move |_, handle: i32| {
                    net_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?
                        .remove(&handle);

                    Ok(())
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
    pub fn create_env(&self) -> Result<LuaTable, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 4)?;

        env.raw_set("fetch", self.net_fetch.clone())?;
        env.raw_set("open", self.net_open.clone())?;
        env.raw_set("read", self.net_read.clone())?;
        env.raw_set("close", self.net_close.clone())?;

        Ok(env)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn net_fetch() -> anyhow::Result<()> {
//         let api = NetworkAPI::new(Lua::new())?;

//         let response = api.net_fetch.call::<LuaTable>(
//             "https://raw.githubusercontent.com/an-anime-team/anime-games-launcher/refs/heads/next/tests/packages/1/package.json"
//         )?;

//         assert_eq!(response.get::<u16>("status")?, 200);
//         assert!(response.get::<bool>("is_ok")?);
//         assert_eq!(Hash::for_slice(&response.get::<Vec<u8>>("body")?), Hash(9442626994218140953));

//         Ok(())
//     }

//     #[test]
//     fn net_read() -> anyhow::Result<()> {
//         let api = NetworkAPI::new(Lua::new())?;

//         let header = api.net_open.call::<LuaTable>(
//             "https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz"
//         )?;

//         assert_eq!(header.get::<u16>("status")?, 200);
//         assert!(header.get::<bool>("is_ok")?);

//         let handle = header.get::<i32>("handle")?;

//         let mut body_len = 0;

//         while let Some(chunk) = api.net_read.call::<Option<Vec<u8>>>(handle)? {
//             body_len += chunk.len();
//         }

//         assert_eq!(body_len, 9215513);

//         Ok(())
//     }
// }
