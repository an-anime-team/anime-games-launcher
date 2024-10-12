use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mlua::prelude::*;

use reqwest::{Client, Method};

use crate::config::STARTUP_CONFIG;

use super::*;

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

pub struct NetworkAPI<'lua> {
    lua: &'lua Lua,

    net_fetch: LuaFunction<'lua>,
    net_open: LuaFunction<'lua>,
    net_read: LuaFunction<'lua>,
    net_close: LuaFunction<'lua>
}

impl<'lua> NetworkAPI<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, PackagesEngineError> {
        let builder = STARTUP_CONFIG.general.network.builder()?;

        let net_client = Arc::new(builder.build()?);
        let net_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            lua,

            net_fetch: {
                let net_client = net_client.clone();

                lua.create_function(move |lua, (url, options): (LuaString, Option<LuaTable>)| {
                    let url = url.to_string_lossy().to_string();
                    let request = create_request(&net_client, url, options)?;

                    // Perform the request.
                    let response = RUNTIME.block_on(async move {
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

                    let (response, header) = RUNTIME.block_on(async move {
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

                    let chunk = RUNTIME.block_on(async move {
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
            }
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable<'lua>, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 4)?;

        env.set("fetch", self.net_fetch.clone())?;
        env.set("open", self.net_open.clone())?;
        env.set("read", self.net_read.clone())?;
        env.set("close", self.net_close.clone())?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn net_fetch() -> anyhow::Result<()> {
        let lua = Lua::new();
        let api = NetworkAPI::new(&lua)?;

        let response = api.net_fetch.call::<_, LuaTable>(
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
        let api = NetworkAPI::new(&lua)?;

        let header = api.net_open.call::<_, LuaTable>(
            "https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz"
        )?;

        assert_eq!(header.get::<_, u16>("status")?, 200);
        assert!(header.get::<_, bool>("is_ok")?);

        let handle = header.get::<_, u32>("handle")?;

        let mut body_len = 0;

        while let Some(chunk) = api.net_read.call::<_, Option<Vec<u8>>>(handle)? {
            body_len += chunk.len();
        }

        assert_eq!(body_len, 9215513);

        Ok(())
    }
}
