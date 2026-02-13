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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use agl_core::export::network::reqwest::{Client, RequestBuilder, Method};
use agl_core::tasks;

use mlua::prelude::*;

use super::bytes::Bytes;
use super::task_api::{Promise, PromiseValue, TaskOutput, task_output};

fn create_request(
    client: &Client,
    url: impl AsRef<str>,
    options: Option<LuaTable>
) -> Result<RequestBuilder, LuaError> {
    let mut method = String::from("get");

    // Change the request method if provided.
    if let Some(options) = &options {
        method = options.get::<Option<String>>("method")?
            .unwrap_or(String::from("get"));
    }

    let method = match method.to_ascii_lowercase().as_str() {
        "get"     => Method::GET,
        "post"    => Method::POST,
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
        if let Some(headers) = options.get::<Option<LuaTable>>("headers")? {
            for pair in headers.pairs::<String, String>() {
                let (key, value) = pair?;

                request = request.header(key, value);
            }
        }

        if let Some(body) = options.get::<Option<Bytes>>("body")? {
            request = request.body(body.to_vec());
        }
    }

    Ok(request)
}

#[derive(Debug)]
pub struct HttpApi {
    lua: Lua,

    http_fetch: LuaFunction,
    http_open: LuaFunction,
    http_read: LuaFunction,
    http_close: LuaFunction
}

impl HttpApi {
    pub fn new(lua: Lua, client: Client) -> Result<Self, LuaError> {
        let net_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            http_fetch: {
                let client = client.clone();

                lua.create_function(move |lua: &Lua, (url, options): (String, Option<LuaTable>)| {
                    let request = create_request(&client, url, options)?;

                    let value = PromiseValue::from_future(async move {
                        let response = request.send().await
                            .map_err(|err| {
                                LuaError::external(format!("failed to perform request: {err}"))
                            })?;

                        let status = response.status();
                        let headers = response.headers().clone();

                        let body = response.bytes().await
                            .map_err(|err| {
                                LuaError::external(format!("failed to fetch body: {err}"))
                            })?
                            .to_vec();

                        Ok(Box::new(move |lua: &Lua| {
                            let headers_table = lua.create_table_with_capacity(0, headers.len())?;

                            for (key, value) in headers {
                                if let Some(key) = key {
                                    headers_table.raw_set(
                                        key.to_string(),
                                        lua.create_string(value.as_bytes())?
                                    )?;
                                }
                            }

                            let body = Bytes::new(body.into_boxed_slice());

                            let result = lua.create_table_with_capacity(0, 4)?;

                            result.raw_set("status", status.as_u16())?;
                            result.raw_set("is_ok", status.is_success())?;
                            result.raw_set("headers", headers_table)?;
                            result.raw_set("body", body)?;

                            Ok(LuaValue::Table(result))
                        }) as TaskOutput)
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })?
            },

            http_open: {
                let client = client.clone();
                let net_handles = net_handles.clone();

                lua.create_function(move |lua: &Lua, (url, options): (String, Option<LuaTable>)| {
                    let request = create_request(&client, url, options)?;

                    let net_handles = net_handles.clone();

                    let value = PromiseValue::from_future(async move {
                        let response = request.send().await
                            .map_err(|err| {
                                LuaError::external(format!("failed to perform request: {err}"))
                            })?;

                        let status = response.status();
                        let headers = response.headers().clone();

                        let mut handles = net_handles.lock()
                            .map_err(|err| {
                                LuaError::external(format!("failed to register handle: {err}"))
                            })?;

                        let mut handle = rand::random::<i32>();

                        while handles.contains_key(&handle) {
                            handle = rand::random::<i32>();
                        }

                        handles.insert(handle, response);

                        Ok(Box::new(move |lua: &Lua| {
                            let headers_table = lua.create_table_with_capacity(0, headers.len())?;

                            for (key, value) in headers {
                                if let Some(key) = key {
                                    headers_table.raw_set(
                                        key.to_string(),
                                        lua.create_string(value.as_bytes())?
                                    )?;
                                }
                            }

                            let result = lua.create_table_with_capacity(0, 4)?;

                            result.raw_set("status", status.as_u16())?;
                            result.raw_set("is_ok", status.is_success())?;
                            result.raw_set("headers", headers_table)?;
                            result.raw_set("handle", handle)?;

                            Ok(LuaValue::Table(result))
                        }) as TaskOutput)
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })?
            },

            http_read: {
                let net_handles = net_handles.clone();

                lua.create_function(move |lua: &Lua, handle: i32| {
                    let net_handles = net_handles.clone();

                    let value = PromiseValue::from_blocking(move || {
                        let mut handles = net_handles.lock()
                            .map_err(|err| {
                                LuaError::external(format!("failed to read handle: {err}"))
                            })?;

                        let Some(response) = handles.get_mut(&handle) else {
                            return Err(LuaError::external("invalid request handle"));
                        };

                        // Blocking instead of a future because there's some
                        // problems with `Send` trait.
                        let chunk = tasks::block_on(response.chunk())
                            .map_err(|err| {
                                LuaError::external(format!("failed to read body chunk: {err}"))
                            })?;

                        let Some(chunk) = chunk else {
                            return Ok(task_output(Ok(LuaValue::Nil)));
                        };

                        let chunk = Bytes::from(chunk.to_vec());

                        Ok(Box::new(move |lua: &Lua| {
                            chunk.into_lua(lua)
                        }) as TaskOutput)
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })?
            },

            http_close: {
                let net_handles = net_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    net_handles.lock()
                        .map_err(|err| {
                            LuaError::external(format!("failed to read handle: {err}"))
                        })?
                        .remove(&handle);

                    Ok(())
                })?
            },

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 4)?;

        env.raw_set("fetch", &self.http_fetch)?;
        env.raw_set("open", &self.http_open)?;
        env.raw_set("read", &self.http_read)?;
        env.raw_set("close", &self.http_close)?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch() -> Result<(), LuaError> {
        let api = HttpApi::new(Lua::new(), Client::new())?;

        let promise = api.http_fetch.call::<LuaAnyUserData>(
            "https://raw.githubusercontent.com/an-anime-team/anime-games-launcher/refs/heads/next/crates/agl-runtime/tests/simple_package/package.json"
        )?;

        let response = promise.call_method::<LuaTable>("await", ())?;

        assert_eq!(response.get::<u16>("status")?, 200);
        assert!(response.get::<bool>("is_ok")?);
        assert_eq!(seahash::hash(&response.get::<Bytes>("body")?), 8997647943168728036);

        Ok(())
    }

    #[test]
    fn read() -> Result<(), LuaError> {
        let api = HttpApi::new(Lua::new(), Client::new())?;

        let promise = api.http_open.call::<LuaAnyUserData>(
            "https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz"
        )?;

        let header = promise.call_method::<LuaTable>("await", ())?;

        assert_eq!(header.get::<u16>("status")?, 200);
        assert!(header.get::<bool>("is_ok")?);

        let handle = header.get::<i32>("handle")?;

        let mut body_len = 0;

        loop {
            let promise = api.http_read.call::<LuaAnyUserData>(handle)?;

            let Some(chunk) = promise.call_method::<Option<Bytes>>("await", ())? else {
                break;
            };

            body_len += chunk.len();
        }

        assert_eq!(body_len, 9215513);

        Ok(())
    }
}
