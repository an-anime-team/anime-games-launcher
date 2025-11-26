use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mlua::prelude::*;

use rusqlite::{Connection, ToSql};
use rusqlite::types::{ValueRef, ToSqlOutput, FromSql, FromSqlResult};

use super::*;

/// Lua to SQLite types bridge.
#[derive(Debug, Clone)]
enum SqliteParam {
    String(String),
    Double(f64),
    Integer(i32),
    Boolean(bool),
    Blob(Vec<u8>),
    Nil
}

impl SqliteParam {
    pub fn to_lua(&self, lua: &Lua) -> Result<LuaValue, LuaError> {
        match self {
            Self::String(value) => lua.create_string(value)
                .map(LuaValue::String),

            Self::Double(value)  => Ok(LuaValue::Number(*value)),
            Self::Integer(value) => Ok(LuaValue::Integer(*value)),
            Self::Boolean(value) => Ok(LuaValue::Boolean(*value)),
            Self::Nil            => Ok(LuaNil),

            Self::Blob(blob) => {
                let result = lua.create_table_with_capacity(blob.len(), 0)?;

                for byte in blob {
                    result.raw_push(*byte)?;
                }

                Ok(LuaValue::Table(result))
            }
        }
    }

    pub fn from_lua(value: &LuaValue) -> Result<Self, LuaError> {
        match value {
            LuaValue::String(value)  => Ok(Self::String(value.to_string_lossy().to_string())),
            LuaValue::Number(value)  => Ok(Self::Double(*value)),
            LuaValue::Integer(value) => Ok(Self::Integer(*value)),
            LuaValue::Boolean(value) => Ok(Self::Boolean(*value)),
            LuaValue::Nil            => Ok(Self::Nil),

            LuaValue::Table(table) => {
                let mut blob = Vec::with_capacity(table.raw_len());

                for byte in table.clone().sequence_values::<u8>() {
                    blob.push(byte?);
                }

                Ok(Self::Blob(blob))
            }

            _ => Err(LuaError::external("can't coerce given value type"))
        }
    }
}

impl ToSql for SqliteParam {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let value = match self {
            Self::String(string)   => ValueRef::Text(string.as_bytes()),
            Self::Double(double)   => ValueRef::Real(*double),
            Self::Integer(integer) => ValueRef::Integer(*integer as i64),
            Self::Boolean(bool)    => ValueRef::Integer(if *bool { 1 } else { 0 }),
            Self::Blob(blob)       => ValueRef::Blob(blob.as_slice()),
            Self::Nil              => ValueRef::Null
        };

        Ok(ToSqlOutput::Borrowed(value))
    }
}

impl FromSql for SqliteParam {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value {
            ValueRef::Text(text)       => Ok(Self::String(String::from_utf8_lossy(text).to_string())),
            ValueRef::Real(real)       => Ok(Self::Double(real)),
            ValueRef::Integer(integer) => Ok(Self::Integer(integer as i32)),
            ValueRef::Blob(blob)       => Ok(Self::Blob(blob.to_vec())),
            ValueRef::Null             => Ok(Self::Nil)
        }
    }
}

pub struct SQLiteAPI {
    lua: Lua,

    sqlite_open: LuaFunctionBuilder,
    sqlite_execute: LuaFunction,
    sqlite_batch: LuaFunction,
    sqlite_query: LuaFunction,
    sqlite_query_row: LuaFunction,
    sqlite_close: LuaFunction
}

impl SQLiteAPI {
    pub fn new(lua: Lua) -> Result<Self, PackagesEngineError> {
        let connection_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            sqlite_open: {
                let connection_handles = connection_handles.clone();

                Box::new(move |lua: &Lua, context: &Context| {
                    let context = context.to_owned();
                    let connection_handles = connection_handles.clone();

                    lua.create_function(move |_, path: LuaString| {
                        let path = resolve_path(path.to_string_lossy())?;

                        if !context.is_accessible(&path) {
                            return Err(LuaError::external("path is inaccessible"));
                        }

                        if let Some(parent) = path.parent() {
                            if !parent.is_dir() {
                                std::fs::create_dir_all(parent)?;
                            }
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

            sqlite_execute: {
                let connection_handles = connection_handles.clone();

                lua.create_function(move |_, (handle, command, params): (i32, LuaString, Option<LuaTable>)| {
                    let mut handles = connection_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(connection) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid database connection handle"));
                    };

                    let mut query = connection.prepare_cached(&command.to_string_lossy())
                        .map_err(LuaError::external)?;

                    // raw_len for better performance.
                    let params_len = params.as_ref()
                        .map(LuaTable::raw_len)
                        .unwrap_or_default();

                    let mut query_params = Vec::with_capacity(params_len);

                    if let Some(params) = params {
                        for param in params.sequence_values::<LuaValue>() {
                            query_params.push(SqliteParam::from_lua(&param?)?);
                        }
                    }

                    query.execute(rusqlite::params_from_iter(query_params))
                        .map_err(LuaError::external)?;

                    Ok(connection.last_insert_rowid())
                })?
            },

            sqlite_batch: {
                let connection_handles = connection_handles.clone();

                lua.create_function(move |_, (handle, command): (i32, LuaString)| {
                    let mut handles = connection_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(connection) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid database connection handle"));
                    };

                    connection.execute_batch(&command.to_string_lossy())
                        .map_err(LuaError::external)?;

                    Ok(())
                })?
            },

            sqlite_query: {
                let connection_handles = connection_handles.clone();

                lua.create_function(move |lua, (handle, query, params): (i32, LuaString, Option<LuaTable>)| -> Result<LuaTable, LuaError> {
                    let mut handles = connection_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(connection) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid database connection handle"));
                    };

                    let mut query = connection.prepare_cached(&query.to_string_lossy())
                        .map_err(LuaError::external)?;

                    // raw_len for better performance.
                    let params_len = params.as_ref()
                        .map(LuaTable::raw_len)
                        .unwrap_or_default();

                    let mut query_params = Vec::with_capacity(params_len);

                    if let Some(params) = params {
                        for param in params.sequence_values::<LuaValue>() {
                            query_params.push(SqliteParam::from_lua(&param?)?);
                        }
                    }

                    let rows = query.query_map(rusqlite::params_from_iter(query_params), |row| {
                        let mut columns = Vec::new();

                        let mut i = 0;

                        while let Ok(column) = row.get::<_, SqliteParam>(i) {
                            columns.push(column);

                            i += 1;
                        }

                        Ok(columns)
                    }).map_err(LuaError::external)?;

                    let result = lua.create_table()?;

                    for row in rows {
                        let row = row.map_err(LuaError::external)?;

                        let result_row = lua.create_table_with_capacity(row.len(), 0)?;

                        for column in row {
                            result_row.raw_push(column.to_lua(lua)?)?;
                        }

                        result.raw_push(result_row)?;
                    }

                    Ok(result)
                })?
            },

            sqlite_query_row: {
                let connection_handles = connection_handles.clone();

                lua.create_function(move |lua, (handle, query, params): (i32, LuaString, Option<LuaTable>)| -> Result<LuaValue, LuaError> {
                    let mut handles = connection_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(connection) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid database connection handle"));
                    };

                    let mut query = connection.prepare_cached(&query.to_string_lossy())
                        .map_err(LuaError::external)?;

                    // raw_len for better performance.
                    let params_len = params.as_ref()
                        .map(LuaTable::raw_len)
                        .unwrap_or_default();

                    let mut query_params = Vec::with_capacity(params_len);

                    if let Some(params) = params {
                        for param in params.sequence_values::<LuaValue>() {
                            query_params.push(SqliteParam::from_lua(&param?)?);
                        }
                    }

                    let row = query.query_row(rusqlite::params_from_iter(query_params), |row| {
                        let mut columns = Vec::new();

                        let mut i = 0;

                        while let Ok(column) = row.get::<_, SqliteParam>(i) {
                            columns.push(column);

                            i += 1;
                        }

                        Ok(columns)
                    });

                    let row = match row {
                        Ok(row) => row,

                        Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(LuaValue::Nil),
                        Err(err) => return Err(LuaError::external(err))
                    };

                    if row.is_empty() {
                        return Ok(LuaValue::Nil);
                    }

                    let result = lua.create_table_with_capacity(row.len(), 0)?;

                    for column in row {
                        result.raw_push(column.to_lua(lua)?)?;
                    }

                    Ok(LuaValue::Table(result))
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

                    connection.execute("PRAGMA optimize", []).map_err(LuaError::external)?;
                    connection.cache_flush().map_err(LuaError::external)?;

                    handles.remove(&handle);

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
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, PackagesEngineError> {
        let env = self.lua.create_table_with_capacity(0, 6)?;

        env.raw_set("open", (self.sqlite_open)(&self.lua, context)?)?;
        env.raw_set("execute", self.sqlite_execute.clone())?;
        env.raw_set("batch", self.sqlite_batch.clone())?;
        env.raw_set("query", self.sqlite_query.clone())?;
        env.raw_set("query_row", self.sqlite_query_row.clone())?;
        env.raw_set("close", self.sqlite_close.clone())?;

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
