// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-runtime
// Copyright (C) 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
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

use std::sync::{Arc, Mutex};

use mlua::prelude::*;

use super::bytes::Bytes;
use super::*;

pub struct SecretsApi {
    lua: Lua,

    secrets_permissions: LuaFunctionBuilder,
    secrets_list: LuaFunctionBuilder,
    secrets_read: LuaFunctionBuilder,
    secrets_write: LuaFunctionBuilder,
    secrets_remove: LuaFunctionBuilder
}

impl SecretsApi {
    pub fn new(lua: Lua, path: impl AsRef<Path>) -> Result<Self, LuaError> {
        let database = rusqlite::Connection::open(path)
            .map_err(LuaError::external)?;

        database.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS secrets_v1 (
                container TEXT NOT NULL,
                key       TEXT NOT NULL,
                value     BLOB NOT NULL,

                PRIMARY KEY (container, key)
            );
        "#).map_err(LuaError::external)?;

        let database = Arc::new(Mutex::new(database));

        Ok(Self {
            secrets_permissions: Box::new(move |lua: &Lua, context: &ModuleContext| {
                let context = context.to_owned();

                lua.create_function(move |lua: &Lua, container: String| {
                    let permissions = lua.create_table_with_capacity(0, 2)?;

                    permissions.raw_set("read", context.can_read_secrets_container(&container))?;
                    permissions.raw_set("write", context.can_write_secrets_container(&container))?;

                    Ok(permissions)
                })
            }),

            secrets_list: {
                let database = database.clone();

                Box::new(move |lua: &Lua, context: &ModuleContext| {
                    let database = database.clone();
                    let context = context.to_owned();

                    lua.create_function(move |lua: &Lua, container: String| {
                        if !context.can_read_secrets_container(&container) {
                            return Err(LuaError::external("no secrets container read permissions"));
                        }

                        let lock = database.lock()
                            .map_err(|_| LuaError::external("failed to open secrets database"))?;

                        let mut query = lock.prepare_cached("SELECT key FROM secrets_v1 WHERE container = ?1")
                            .map_err(|err| {
                                LuaError::external("failed to read secret container entries")
                                    .context(err)
                            })?;

                        let mut entries = query.query([container])
                            .map_err(|err| {
                                LuaError::external("failed to read secret container entries")
                                    .context(err)
                            })?;

                        let result = lua.create_table()?;

                        while let Some(entry) = entries.next()
                            .map_err(|err| {
                                LuaError::external("failed to read secret container entry")
                                    .context(err)
                            })?
                        {
                            let entry_key = entry.get::<_, String>("key")
                                .map_err(|err| {
                                    LuaError::external("failed to read secret container entry")
                                        .context(err)
                                })?;

                            result.raw_push(entry_key)?;
                        }

                        Ok(result)
                    })
                })
            },

            secrets_read: {
                let database = database.clone();

                Box::new(move |lua: &Lua, context: &ModuleContext| {
                    let database = database.clone();
                    let context = context.to_owned();

                    lua.create_function(move |lua: &Lua, (container, key): (String, String)| {
                        if !context.can_read_secrets_container(&container) {
                            return Err(LuaError::external("no secrets container read permissions"));
                        }

                        let lock = database.lock()
                            .map_err(|_| LuaError::external("failed to open secrets database"))?;

                        let value = lock.prepare_cached("SELECT value FROM secrets_v1 WHERE container = ?1 AND key = ?2")
                            .and_then(|mut query| {
                                query.query_row(
                                    [container, key],
                                    |row| row.get::<_, Box<[u8]>>("value")
                                )
                            })
                            .map(Some)
                            .or_else(|err| {
                                if err == rusqlite::Error::QueryReturnedNoRows {
                                    Ok(None)
                                } else {
                                    Err(err)
                                }
                            })
                            .map_err(|err| {
                                LuaError::external("failed to read secret container entry")
                                    .context(err)
                            })?
                            .map(|value| value.into_lua(lua))
                            .transpose()?;

                        Ok(value)
                    })
                })
            },

            secrets_write: {
                let database = database.clone();

                Box::new(move |lua: &Lua, context: &ModuleContext| {
                    let database = database.clone();
                    let context = context.to_owned();

                    lua.create_function(move |_lua: &Lua, (container, key, value): (String, String, Bytes)| {
                        if !context.can_write_secrets_container(&container) {
                            return Err(LuaError::external("no secrets container write permissions"));
                        }

                        let lock = database.lock()
                            .map_err(|_| LuaError::external("failed to open secrets database"))?;

                        lock.prepare_cached("INSERT OR REPLACE INTO secrets_v1 (container, key, value) VALUES (?1, ?2, ?3)")
                            .and_then(|mut query| {
                                query.execute((
                                    container,
                                    key,
                                    Box::<[u8]>::from(value)
                                ))
                            })
                            .map_err(|err| {
                                LuaError::external("failed to write secret container entry")
                                    .context(err)
                            })?;

                        Ok(())
                    })
                })
            },

            secrets_remove: {
                let database = database.clone();

                Box::new(move |lua: &Lua, context: &ModuleContext| {
                    let database = database.clone();
                    let context = context.to_owned();

                    lua.create_function(move |_lua: &Lua, (container, key): (String, Option<String>)| {
                        if !context.can_write_secrets_container(&container) {
                            return Err(LuaError::external("no secrets container write permissions"));
                        }

                        let lock = database.lock()
                            .map_err(|_| LuaError::external("failed to open secrets database"))?;

                        let result = if let Some(key) = key {
                            lock.prepare_cached("DELETE FROM secrets_v1 WHERE container = ?1 AND key = ?2")
                                .and_then(|mut query| {
                                    query.execute([container, key])
                                })
                        }

                        else {
                            lock.prepare_cached("DELETE FROM secrets_v1 WHERE container = ?1")
                                .and_then(|mut query| {
                                    query.execute([container])
                                })
                        };

                        result.map_err(|err| {
                            LuaError::external("failed to query secret container entry")
                                .context(err)
                        })?;

                        Ok(())
                    })
                })
            },

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(
        &self,
        context: &ModuleContext
    ) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 5)?;

        env.raw_set("permissions", (self.secrets_permissions)(&self.lua, context)?)?;
        env.raw_set("list", (self.secrets_list)(&self.lua, context)?)?;
        env.raw_set("read", (self.secrets_read)(&self.lua, context)?)?;
        env.raw_set("write", (self.secrets_write)(&self.lua, context)?)?;
        env.raw_set("remove", (self.secrets_remove)(&self.lua, context)?)?;

        Ok(env)
    }
}

#[test]
fn test_secrets() -> Result<(), LuaError> {
    let path = std::env::temp_dir().join(".agl-secrets-test.db");

    if path.exists() {
        std::fs::remove_file(&path)?;
    }

    let api = SecretsApi::new(Lua::new(), &path)?;

    let env = api.create_env(&ModuleContext {
        temp_dir: Arc::new(std::env::temp_dir()),
        module_dir: Arc::new(std::env::temp_dir()),
        persistent_dir: Arc::new(std::env::temp_dir()),
        scope: Arc::new(RwLock::new(ModuleScope {
            secrets_read_containers: vec![
                String::from("read"),
                String::from("write")
            ],
            secrets_write_containers: vec![
                String::from("write")
            ],

            ..ModuleScope::default()
        }))
    })?;

    // Validate secrets.permissions API
    let permissions = env.call_function::<LuaTable>("permissions", "read")?;

    assert!(permissions.raw_get::<bool>("read")?);
    assert!(!permissions.raw_get::<bool>("write")?);

    let permissions = env.call_function::<LuaTable>("permissions", "write")?;

    assert!(permissions.raw_get::<bool>("read")?);
    assert!(permissions.raw_get::<bool>("write")?);

    let permissions = env.call_function::<LuaTable>("permissions", "unknown")?;

    assert!(!permissions.raw_get::<bool>("read")?);
    assert!(!permissions.raw_get::<bool>("write")?);

    // Validate read-container permissions and entries list.
    assert!(env.call_function::<Vec<String>>("list", "unknown").is_err());
    assert!(env.call_function::<Vec<String>>("list", "read")?.is_empty());
    assert!(env.call_function::<Vec<String>>("list", "write")?.is_empty());

    assert!(env.call_function::<()>("write", ("unknown", "test", "hello")).is_err());
    assert!(env.call_function::<()>("write", ("read", "test", "hello")).is_err());

    env.call_function::<()>("write", ("write", "test", "hello"))?;

    assert_eq!(env.call_function::<Vec<String>>("list", "write")?, ["test"]);

    // Validate write-container read and remove.
    assert!(env.call_function::<Option<Bytes>>("read", ("unknown", "test", "hello")).is_err());
    assert!(env.call_function::<Option<Bytes>>("read", ("read", "test", "hello"))?.is_none());

    assert_eq!(
        env.call_function::<Option<Bytes>>("read", ("write", "test"))?
            .as_ref()
            .map(|buf| buf.as_slice()),
        Some("hello".as_bytes())
    );

    env.call_function::<()>("remove", "write")?;

    assert!(env.call_function::<Option<Bytes>>("read", ("write", "test"))?.is_none());

    std::fs::remove_file(path)?;

    Ok(())
}
