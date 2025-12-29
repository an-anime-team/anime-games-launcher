// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-runtime
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
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

use std::sync::{Mutex, MutexGuard};

use mlua::prelude::*;

use agl_core::tasks::{self, JoinHandle};

/// Inner value of a promise. Exists because promise can mutate its stored value
/// on the fly.
#[derive(Debug)]
pub enum PromiseValue {
    Value(LuaValue),
    Callback(LuaFunction),
    Coroutine(LuaThread),
    Task(JoinHandle<Result<LuaValue, LuaError>>)
}

impl PromiseValue {
    pub fn from_lua_value(value: LuaValue) -> Self {
        match value {
            LuaValue::Function(callback) => Self::Callback(callback),
            LuaValue::Thread(coroutine) => Self::Coroutine(coroutine),
            _ => Self::Value(value)
        }
    }

    pub fn from_future(
        future: impl Future<Output = Result<LuaValue, LuaError>> + Send + 'static
    ) -> Self {
        Self::Task(tasks::spawn(future))
    }
}

/// A lua usertype wrapper over a promise value. Implements `poll` method to
/// query output value.
#[derive(Default, Debug)]
pub struct Promise(Mutex<Option<PromiseValue>>);

impl Promise {
    pub fn new(value: PromiseValue) -> Self {
        Self(Mutex::new(Some(value)))
    }

    pub fn from_lua_value(value: LuaValue) -> Self {
        Self::new(PromiseValue::from_lua_value(value))
    }

    #[inline]
    pub fn lock(&self) -> MutexGuard<'_, Option<PromiseValue>> {
        self.0.lock().expect("failed to lock promise value")
    }
}

impl LuaUserData for Promise {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("poll", |lua: &Lua, promise: &Self, _: ()| -> Result<LuaMultiValue, LuaError> {
            let mut lock = promise.lock();

            let Some(value) = lock.take() else {
                return Err(LuaError::external("task already finished"));
            };

            match value {
                PromiseValue::Value(value) => {
                    lua.pack_multi((true, value))
                }

                PromiseValue::Callback(callback) => {
                    let (status, value) = callback.call::<(Option<bool>, LuaValue)>(())?;

                    // Do not execute function if it's finished or aborted.
                    if status == Some(false) {
                        *lock = Some(PromiseValue::Callback(callback));
                    }

                    lua.pack_multi((status, value))
                }

                PromiseValue::Coroutine(coroutine) => {
                    let value = coroutine.resume::<LuaValue>(())?;

                    match coroutine.status() {
                        LuaThreadStatus::Finished => {
                            lua.pack_multi((true, value))
                        }

                        LuaThreadStatus::Resumable | LuaThreadStatus::Running => {
                            *lock = Some(PromiseValue::Coroutine(coroutine));

                            lua.pack_multi((false, value))
                        }

                        LuaThreadStatus::Error => {
                            lua.pack_multi((LuaValue::Nil, value))
                        }
                    }
                }

                PromiseValue::Task(handle) => {
                    if handle.is_finished() {
                        let value = tasks::block_on(handle)
                            .map_err(|err| {
                                LuaError::external(format!("failed to execute task: {err}"))
                            })?;

                        lua.pack_multi((true, value))
                    }

                    else {
                        *lock = Some(PromiseValue::Task(handle));

                        lua.pack_multi((false, LuaValue::Nil))
                    }
                }
            }
        });

        methods.add_method("abort", |_, promise: &Self, _: ()| -> Result<(), LuaError> {
            let mut lock = promise.lock();

            let Some(value) = lock.take() else {
                return Ok(());
            };

            if let PromiseValue::Task(handle) = value {
                handle.abort();
            }

            Ok(())
        });
    }

    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("finished", |_, promise: &Self| -> Result<bool, LuaError> {
            Ok(promise.lock().is_some())
        });

        fields.add_field_method_get("background", |_, promise: &Self| -> Result<bool, LuaError> {
            Ok(matches!(&*promise.lock(), Some(PromiseValue::Task(_))))
        });
    }
}

impl Drop for Promise {
    fn drop(&mut self) {
        let mut lock = self.lock();

        let Some(value) = lock.take() else {
            return;
        };

        if let PromiseValue::Task(handle) = value {
            handle.abort();
        }
    }
}

pub struct TaskApi {
    lua: Lua,

    task_create: LuaFunction
}

impl TaskApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        Ok(Self {
            task_create: lua.create_function(|lua: &Lua, task: LuaValue| {
                Promise::from_lua_value(task).into_lua(lua)
            })?,

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 1)?;

        env.raw_set("create", &self.task_create)?;

        Ok(env)
    }
}
