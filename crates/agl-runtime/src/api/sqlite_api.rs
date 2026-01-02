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

use mlua::prelude::*;

use rusqlite::{Connection, ToSql};
use rusqlite::types::{ValueRef, ToSqlOutput, FromSql, FromSqlResult};

use super::bytes::Bytes;
use super::task_api::{Promise, PromiseValue, TaskOutput, task_output};
use super::*;

/// Lua to SQLite types bridge.
#[derive(Debug, Clone)]
enum SqliteParam {
    String(String),
    Double(f64),
    Integer(i64),
    Boolean(bool),
    Blob(Bytes),
    Null
}

impl FromLua for SqliteParam {
    fn from_lua(value: LuaValue, lua: &Lua) -> Result<Self, LuaError> {
        match value {
            LuaValue::String(value)  => Ok(Self::String(value.to_string_lossy().to_string())),
            LuaValue::Number(value)  => Ok(Self::Double(value)),
            LuaValue::Integer(value) => Ok(Self::Integer(value)),
            LuaValue::Boolean(value) => Ok(Self::Boolean(value)),
            LuaValue::Nil            => Ok(Self::Null),

            // Try to use table sequence values as bytes blob.
            LuaValue::Table(_) => Bytes::from_lua(value, lua).map(Self::Blob),

            // Decode param from lua function output.
            LuaValue::Function(callback) => {
                Self::from_lua(callback.call::<LuaValue>(())?, lua)
            }

            // Decode param from a Promise result.
            LuaValue::UserData(ref object) if object.get::<Option<LuaFunction>>("await")?.is_some() => {
                let value = object.call_method::<LuaValue>("await", ())?;

                Self::from_lua(value, lua)
            }

            // Use Bytes as Blob.
            LuaValue::UserData(ref object) if object.get::<Option<LuaFunction>>("as_table")?.is_some() => {
                Bytes::from_lua(value, lua).map(Self::Blob)
            }

            _ => Err(LuaError::external("can't coerce given value type"))
        }
    }
}

impl IntoLua for SqliteParam {
    fn into_lua(self, lua: &Lua) -> Result<LuaValue, LuaError> {
        match self {
            Self::String(str) => lua.create_string(str)
                .map(LuaValue::String),

            Self::Double(double)   => Ok(LuaValue::Number(double)),
            Self::Integer(integer) => Ok(LuaValue::Integer(integer)),
            Self::Boolean(bool)    => Ok(LuaValue::Boolean(bool)),
            Self::Null             => Ok(LuaValue::Nil),

            Self::Blob(bytes) => bytes.into_lua(lua)
        }
    }
}

impl ToSql for SqliteParam {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let value = match self {
            Self::String(string)   => ValueRef::Text(string.as_bytes()),
            Self::Double(double)   => ValueRef::Real(*double),
            Self::Integer(integer) => ValueRef::Integer(*integer),
            Self::Boolean(bool)    => ValueRef::Integer(if *bool { 1 } else { 0 }),
            Self::Blob(blob)       => ValueRef::Blob(blob.as_slice()),
            Self::Null             => ValueRef::Null
        };

        Ok(ToSqlOutput::Borrowed(value))
    }
}

impl FromSql for SqliteParam {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(text)       => Ok(Self::String(String::from_utf8_lossy(text).to_string())),
            ValueRef::Real(real)       => Ok(Self::Double(real)),
            ValueRef::Integer(integer) => Ok(Self::Integer(integer)),
            ValueRef::Blob(blob)       => Ok(Self::Blob(Bytes::from(blob.to_vec()))),
            ValueRef::Null             => Ok(Self::Null)
        }
    }
}

pub struct SqliteApi {
    lua: Lua,

    sqlite_open: LuaFunctionBuilder,
    sqlite_exec: LuaFunction,
    sqlite_batch: LuaFunction,
    sqlite_query: LuaFunction,
    sqlite_query_row: LuaFunction,
    sqlite_close: LuaFunction
}

impl SqliteApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        let connection_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            sqlite_open: {
                let connection_handles = connection_handles.clone();

                Box::new(move |lua: &Lua, context: &Context| {
                    let context = context.to_owned();
                    let connection_handles = connection_handles.clone();

                    lua.create_function(move |_, mut path: PathBuf| {
                        if path.is_relative() {
                            path = context.module_folder.join(path);
                        }

                        path = normalize_path(path, true)
                            .map_err(|err| {
                                LuaError::external(format!("failed to normalize path: {err}"))
                            })?;

                        if !context.can_write_path(&path)? {
                            return Err(LuaError::external("no path write permissions"));
                        }

                        if let Some(parent) = path.parent() && !parent.is_dir() {
                            std::fs::create_dir_all(parent)?;
                        }

                        let connection = Connection::open(path)
                            .map_err(LuaError::external)?;

                        let mut handles = connection_handles.lock()
                            .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                        let mut handle = rand::random::<i32>();

                        while handles.contains_key(&handle) {
                            handle = rand::random::<i32>();
                        }

                        handles.insert(handle, connection);

                        Ok(handle)
                    })
                })
            },

            sqlite_exec: {
                let connection_handles = connection_handles.clone();

                lua.create_function(move |lua: &Lua, (handle, command, params): (i32, LuaString, Option<LuaTable>)| {
                    let connection_handles = connection_handles.clone();

                    let command = command.to_string_lossy()
                        .to_string();

                    let mut query_params = vec![];

                    if let Some(params) = params {
                        query_params = params.sequence_values::<SqliteParam>()
                            .collect::<Result<_, LuaError>>()?;
                    }

                    let value = PromiseValue::from_blocking(move || {
                        let mut handles = connection_handles.lock()
                            .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                        let Some(connection) = handles.get_mut(&handle) else {
                            return Err(LuaError::external("invalid database connection handle"));
                        };

                        let mut query = connection.prepare_cached(&command)
                            .map_err(LuaError::external)?;

                        query.execute(rusqlite::params_from_iter(query_params))
                            .map_err(LuaError::external)?;

                        Ok(task_output(Ok(LuaValue::Integer(connection.last_insert_rowid()))))
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })?
            },

            sqlite_batch: {
                let connection_handles = connection_handles.clone();

                lua.create_function(move |lua: &Lua, (handle, command): (i32, LuaString)| {
                    let connection_handles = connection_handles.clone();

                    let command = command.to_string_lossy()
                        .to_string();

                    let value = PromiseValue::from_blocking(move || {
                        let mut handles = connection_handles.lock()
                            .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                        let Some(connection) = handles.get_mut(&handle) else {
                            return Err(LuaError::external("invalid database connection handle"));
                        };

                        connection.execute_batch(&command)
                            .map_err(LuaError::external)?;

                        Ok(task_output(Ok(LuaValue::Nil)))
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })?
            },

            sqlite_query: {
                let connection_handles = connection_handles.clone();

                lua.create_function(move |lua: &Lua, (handle, query, params): (i32, LuaString, Option<LuaTable>)| {
                    let connection_handles = connection_handles.clone();

                    let query = query.to_string_lossy()
                        .to_string();

                    let mut query_params = vec![];

                    if let Some(params) = params {
                        query_params = params.sequence_values::<SqliteParam>()
                            .collect::<Result<_, LuaError>>()?;
                    }

                    let value = PromiseValue::from_blocking(move || {
                        let mut handles = connection_handles.lock()
                            .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                        let Some(connection) = handles.get_mut(&handle) else {
                            return Err(LuaError::external("invalid database connection handle"));
                        };

                        let mut query = connection.prepare_cached(&query)
                            .map_err(LuaError::external)?;

                        let rows = query.query_map(rusqlite::params_from_iter(query_params), |row| {
                            let mut columns = Vec::new();

                            let mut i = 0;

                            while let Ok(column) = row.get::<_, SqliteParam>(i) {
                                columns.push(column);

                                i += 1;
                            }

                            Ok(columns)
                        }).map_err(LuaError::external)?;

                        let rows = rows.collect::<Result<Vec<Vec<SqliteParam>>, rusqlite::Error>>()
                            .map_err(LuaError::external)?;

                        Ok(Box::new(move |lua: &Lua| {
                            let result = lua.create_table_with_capacity(rows.len(), 0)?;

                            for row in rows {
                                let row = row.into_iter()
                                    .map(|column| column.into_lua(lua))
                                    .collect::<Result<Box<[_]>, _>>()?;

                                result.raw_push(lua.create_sequence_from(row)?)?;
                            }

                            Ok(LuaValue::Table(result))
                        }) as TaskOutput)
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })?
            },

            sqlite_query_row: {
                let connection_handles = connection_handles.clone();

                lua.create_function(move |lua: &Lua, (handle, query, params): (i32, LuaString, Option<LuaTable>)| -> Result<LuaValue, LuaError> {
                    let connection_handles = connection_handles.clone();

                    let query = query.to_string_lossy()
                        .to_string();

                    let mut query_params = vec![];

                    if let Some(params) = params {
                        query_params = params.sequence_values::<SqliteParam>()
                            .collect::<Result<_, LuaError>>()?;
                    }

                    let value = PromiseValue::from_blocking(move || {
                        let mut handles = connection_handles.lock()
                            .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                        let Some(connection) = handles.get_mut(&handle) else {
                            return Err(LuaError::external("invalid database connection handle"));
                        };

                        let mut query = connection.prepare_cached(&query)
                            .map_err(LuaError::external)?;

                        let row = query.query_row(rusqlite::params_from_iter(query_params), |row| {
                            let mut columns = Vec::new();

                            let mut i = 0;

                            while let Ok(column) = row.get::<_, SqliteParam>(i) {
                                columns.push(column);

                                i += 1;
                            }

                            Ok(columns)
                        });

                        Ok(Box::new(move |lua: &Lua| {
                            let row = match row {
                                Ok(row) => row,

                                Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(LuaValue::Nil),
                                Err(err) => return Err(LuaError::external(err))
                            };

                            if row.is_empty() {
                                return Ok(LuaValue::Nil);
                            }

                            let row = row.into_iter()
                                .map(|column| column.into_lua(lua))
                                .collect::<Result<Vec<LuaValue>, LuaError>>()?;

                            lua.create_sequence_from(row)
                                .map(LuaValue::Table)
                        }) as TaskOutput)
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })?
            },

            sqlite_close: {
                let connection_handles = connection_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    let mut handles = connection_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(connection) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid database connection handle"));
                    };

                    connection.execute("PRAGMA optimize", [])
                        .map_err(LuaError::external)?;

                    connection.cache_flush()
                        .map_err(LuaError::external)?;

                    handles.remove(&handle);

                    Ok(())
                })?
            },

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 6)?;

        env.raw_set("open", (self.sqlite_open)(&self.lua, context)?)?;
        env.raw_set("exec", &self.sqlite_exec)?;
        env.raw_set("batch", &self.sqlite_batch)?;
        env.raw_set("query", &self.sqlite_query)?;
        env.raw_set("query_row", &self.sqlite_query_row)?;
        env.raw_set("close", &self.sqlite_close)?;

        Ok(env)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn sqlite_queries() -> anyhow::Result<()> {
//         let path = std::env::temp_dir().join(".agl-v1-sqlite-queries-test.db");

//         if path.exists() {
//             std::fs::remove_file(&path)?;
//         }

//         let path = path.to_string_lossy().to_string();

//         let lua = Lua::new();
//         let api = SQLiteAPI::new(lua.clone())?;

//         let env = api.create_env(&Context {
//             resource_hash: Hash::rand(),
//             temp_folder: std::env::temp_dir(),
//             module_folder: std::env::temp_dir(),
//             persistent_folder: std::env::temp_dir(),
//             input_resources: vec![],
//             ext_process_api: false,
//             ext_allowed_paths: vec![],
//             local_validator: LocalValidator::open(std::env::temp_dir().join("local_validator.json"))?
//         })?;

//         let handle = env.call_function::<i32>("open", path.to_string())?;

//         env.call_function::<()>("execute", (handle, "
//             CREATE TABLE test (
//                 id    INTEGER UNIQUE NOT NULL,
//                 value TEXT NOT NULL,

//                 PRIMARY KEY (id)
//             );
//         "))?;

//         let row_1 = env.call_function::<i64>("execute", (handle, "INSERT INTO test (id, value) VALUES (5, 'test 1')"))?;
//         let row_2 = env.call_function::<i64>("execute", (handle, "INSERT INTO test (id, value) VALUES (?1, 'test 2')", [10]))?;
//         let row_3 = env.call_function::<i64>("execute", (handle, "INSERT INTO test (id, value) VALUES (15, ?1)", ["test 3"]))?;
//         let row_4 = env.call_function::<i64>("execute", (handle, "INSERT INTO test (id, value) VALUES (?1, ?2)", [LuaValue::Integer(20), LuaValue::String(lua.create_string("test 4")?)]))?;

//         assert_eq!(row_1, 5);
//         assert_eq!(row_2, 10);
//         assert_eq!(row_3, 15);
//         assert_eq!(row_4, 20);

//         let rows_count = env.call_function::<LuaTable>("query_row", (handle, "SELECT COUNT(id) FROM test"))?;

//         assert_eq!(rows_count.pop::<i32>()?, 4);

//         let rows = env.call_function::<Vec<LuaTable>>("query", (handle, "SELECT value FROM test WHERE id > ?1", [0]))?;

//         for row in rows {
//             assert!(row.pop::<String>()?.starts_with("test "));
//         }

//         env.call_function::<()>("batch", (handle, "
//             BEGIN TRANSACTION;
//                 DELETE FROM test WHERE id = 5;
//                 DELETE FROM test WHERE id = 10;
//                 DELETE FROM test WHERE id = 15;
//                 DELETE FROM test WHERE id = 20;
//             COMMIT;
//         "))?;

//         let rows_count = env.call_function::<LuaTable>("query_row", (handle, "SELECT COUNT(id) FROM test"))?;

//         assert_eq!(rows_count.pop::<i32>()?, 0);

//         env.call_function::<()>("close", handle)?;

//         std::fs::remove_file(path)?;

//         Ok(())
//     }
// }
