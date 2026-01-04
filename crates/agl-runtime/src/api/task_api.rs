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

use std::sync::{Mutex, MutexGuard};

use mlua::prelude::*;

use agl_core::tasks::{self, JoinHandle};

// I had to do this because if we block a lua engine thread to await some
// promise using either `promise:await()` or `await(promise)` we won't be able
// to create an output value in the engine from the rust side (e.g.
// `lua.create_table()` will just block the rust side because the lua side is
// also blocked by the `await` call). This forces us to return a rust callback
// to the promise which could make the output value itself using its own lua
// engine reference.
pub type TaskOutput = Box<dyn FnOnce(&Lua) -> Result<LuaValue, LuaError> + Send + 'static>;

/// Create new `TaskOutput` if output value is already known.
#[inline]
pub fn task_output(result: Result<LuaValue, LuaError>) -> TaskOutput {
    Box::new(move |_: &Lua| result) as TaskOutput
}

/// Inner value of a promise. Exists because promise can mutate its stored value
/// on the fly.
pub enum PromiseValue {
    Value(LuaValue),
    Callback(LuaFunction),
    Coroutine(LuaThread),
    LuaPromise(LuaAnyUserData),
    Task(JoinHandle<Result<TaskOutput, LuaError>>),
    AnyTask(Box<[Promise]>)
}

impl std::fmt::Debug for PromiseValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(value) => f.debug_struct("PromiseValue")
                .field("value", &value)
                .finish(),

            Self::Callback(callback) => f.debug_struct("PromiseValue")
                .field("callback", &callback)
                .finish(),

            Self::Coroutine(coroutine) => f.debug_struct("PromiseValue")
                .field("coroutine", &coroutine)
                .finish(),

            Self::LuaPromise(promise) => f.debug_struct("PromiseValue")
                .field("promise", &promise)
                .finish(),

            Self::Task(handle) => f.debug_struct("PromiseValue")
                .field("handle", &handle.id())
                .finish(),

            Self::AnyTask(tasks) => f.debug_struct("PromiseValue")
                .field("tasks", &tasks)
                .finish()
        }
    }
}

impl PromiseValue {
    pub fn from_lua_value(value: LuaValue) -> Self {
        match value {
            LuaValue::Function(callback) => Self::Callback(callback),
            LuaValue::Thread(coroutine) => Self::Coroutine(coroutine),

            LuaValue::UserData(object) => {
                if object.type_name().ok().flatten().as_deref() == Some("Promise") {
                    Self::LuaPromise(object)
                } else {
                    Self::Value(LuaValue::UserData(object))
                }
            }

            _ => Self::Value(value)
        }
    }

    pub fn from_future(
        future: impl Future<Output = Result<TaskOutput, LuaError>> + Send + 'static
    ) -> Self {
        Self::Task(tasks::spawn(future))
    }

    pub fn from_blocking(
        callback: impl FnOnce() -> Result<TaskOutput, LuaError> + Send + 'static
    ) -> Self {
        Self::Task(tasks::spawn_blocking(callback))
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
    fn lock(&self) -> MutexGuard<'_, Option<PromiseValue>> {
        self.0.lock().expect("failed to lock promise value")
    }
}

impl FromLua for Promise {
    fn from_lua(value: LuaValue, _: &Lua) -> LuaResult<Self> {
        Ok(Self::from_lua_value(value))
    }
}

impl LuaUserData for Promise {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("finished", |_, promise: &Self| -> Result<bool, LuaError> {
            Ok(promise.lock().is_some())
        });

        fields.add_field_method_get("background", |_, promise: &Self| -> Result<bool, LuaError> {
            Ok(matches!(&*promise.lock(), Some(PromiseValue::Task(_))))
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        fn poll(lua: &Lua, promise: &Promise) -> Result<(Option<bool>, LuaValue), LuaError> {
            let mut lock = promise.lock();

            let Some(value) = lock.take() else {
                return Err(LuaError::external("task already finished"));
            };

            match value {
                PromiseValue::Value(value) => Ok((Some(true), value)),

                PromiseValue::Callback(callback) => {
                    let mut values = callback.call::<LuaVariadic<LuaValue>>(())?;

                    match values.len() {
                        0 => Ok((Some(true), LuaValue::Nil)),

                        1 => {
                            let Some(value) = values.pop() else {
                                return Err(LuaError::external("failed to take callback output"));
                            };

                            Ok((Some(true), value))
                        }

                        2 => {
                            let Some(value) = values.pop() else {
                                return Err(LuaError::external("failed to take callback value"));
                            };

                            let Some(status) = values.pop() else {
                                return Err(LuaError::external("failed to take callback status"));
                            };

                            let status = status.as_boolean();

                            // Do not execute function if it's finished or aborted.
                            if status == Some(false) {
                                *lock = Some(PromiseValue::Callback(callback));
                            }

                            Ok((status, value))
                        }

                        _ => Ok((None, LuaValue::Nil))
                    }
                }

                PromiseValue::Coroutine(coroutine) => {
                    let value = coroutine.resume::<LuaValue>(())?;

                    match coroutine.status() {
                        LuaThreadStatus::Finished => Ok((Some(true), value)),
                        LuaThreadStatus::Error => Ok((None, value)),

                        LuaThreadStatus::Resumable | LuaThreadStatus::Running => {
                            *lock = Some(PromiseValue::Coroutine(coroutine));

                            Ok((Some(false), value))
                        }
                    }
                }

                PromiseValue::LuaPromise(promise) => {
                    let (status, value) = promise.call_method::<(Option<bool>, LuaValue)>("poll", ())?;

                    // Do not execute function if it's finished or aborted.
                    if status == Some(false) {
                        *lock = Some(PromiseValue::LuaPromise(promise));
                    }

                    Ok((status, value))
                }

                PromiseValue::Task(handle) => {
                    if handle.is_finished() {
                        let get_value = tasks::block_on(handle)
                            .map_err(|err| {
                                LuaError::external(format!("failed to execute task: {err}"))
                            })??;

                        Ok((Some(true), get_value(lua)?))
                    }

                    else {
                        *lock = Some(PromiseValue::Task(handle));

                        Ok((Some(false), LuaValue::Nil))
                    }
                }

                PromiseValue::AnyTask(tasks) => {
                    if tasks.is_empty() {
                        return Ok((Some(true), LuaValue::Nil));
                    }

                    for task in &tasks {
                        let (status, value) = poll(lua, task)?;

                        if status != Some(false) {
                            return Ok((status, value));
                        }
                    }

                    *lock = Some(PromiseValue::AnyTask(tasks));

                    Ok((Some(false), LuaValue::Nil))
                }
            }
        }

        methods.add_method("poll", |lua: &Lua, promise: &Self, _: ()| -> Result<LuaMultiValue, LuaError> {
            lua.pack_multi(poll(lua, promise)?)
        });

        methods.add_method("await", |lua: &Lua, promise: &Self, _: ()| -> Result<LuaValue, LuaError> {
            let mut lock = promise.lock();

            let Some(value) = lock.take() else {
                return Err(LuaError::external("task already finished"));
            };

            match value {
                PromiseValue::Value(value) => Ok(value),

                PromiseValue::Callback(callback) => {
                    loop {
                        let mut values = callback.call::<LuaVariadic<LuaValue>>(())?;

                        match values.len() {
                            1 => {
                                let Some(value) = values.pop() else {
                                    return Err(LuaError::external("failed to take callback output"));
                                };

                                return Ok(value);
                            }

                            2 => {
                                let Some(value) = values.pop() else {
                                    return Err(LuaError::external("failed to take callback value"));
                                };

                                let Some(status) = values.pop() else {
                                    return Err(LuaError::external("failed to take callback status"));
                                };

                                let status = status.as_boolean();

                                if status != Some(false) {
                                    return Ok(value);
                                }
                            }

                            _ => return Ok(LuaValue::Nil)
                        }
                    }
                }

                PromiseValue::Coroutine(coroutine) => {
                    loop {
                        let value = coroutine.resume::<LuaValue>(())?;

                        match coroutine.status() {
                            LuaThreadStatus::Finished | LuaThreadStatus::Error => {
                                return Ok(value);
                            }

                            LuaThreadStatus::Resumable | LuaThreadStatus::Running => ()
                        }
                    }
                }

                PromiseValue::LuaPromise(promise) => {
                    promise.call_method("await", ())
                }

                PromiseValue::Task(handle) => {
                    let get_value = tasks::block_on(handle)
                        .map_err(|err| {
                            LuaError::external(format!("failed to execute task: {err}"))
                        })??;

                    get_value(lua)
                }

                PromiseValue::AnyTask(tasks) => {
                    if tasks.is_empty() {
                        return Ok(LuaValue::Nil);
                    }

                    loop {
                        for task in &tasks {
                            let (status, value) = poll(lua, task)?;

                            if status != Some(false) {
                                return Ok(value);
                            }
                        }
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

#[derive(Debug)]
pub struct TaskApi {
    lua: Lua,

    task_create: LuaFunction,
    task_sleep: LuaFunction,
    task_any: LuaFunction
}

impl TaskApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        Ok(Self {
            task_create: lua.create_function(|lua: &Lua, task: LuaValue| {
                Promise::from_lua_value(task)
                    .into_lua(lua)
            })?,

            task_sleep: lua.create_function(|lua: &Lua, (duration, callback): (u32, Option<LuaFunction>)| {
                let duration = std::time::Duration::from_millis(duration as u64);

                let value = PromiseValue::from_future(async move {
                    tasks::sleep(duration).await;

                    let Some(callback) = callback else {
                        return Ok(task_output(Ok(LuaValue::Nil)));
                    };

                    Ok(Box::new(move |_: &Lua| {
                        match callback.call::<LuaValue>(()) {
                            Ok(value) => Ok(value),

                            Err(err) => {
                                #[cfg(feature = "tracing")]
                                tracing::error!(?err, "sleep callback execution error");

                                Err(err)
                            }
                        }
                    }) as TaskOutput)
                });

                Promise::new(value)
                    .into_lua(lua)
            })?,

            task_any: lua.create_function(|lua: &Lua, lua_tasks: LuaVariadic<LuaValue>| {
                if lua_tasks.is_empty() {
                    return Promise::from_lua_value(LuaValue::Nil)
                        .into_lua(lua);
                }

                let promises = lua_tasks.into_iter()
                    .map(Promise::from_lua_value)
                    .collect::<Box<[Promise]>>();

                Promise::new(PromiseValue::AnyTask(promises))
                    .into_lua(lua)
            })?,

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 3)?;

        env.raw_set("create", &self.task_create)?;
        env.raw_set("sleep", &self.task_sleep)?;
        env.raw_set("any", &self.task_any)?;

        Ok(env)
    }
}
