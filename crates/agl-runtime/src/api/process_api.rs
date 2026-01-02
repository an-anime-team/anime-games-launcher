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
use std::io::{Read, Write};
use std::process::{Command, Stdio};

use mlua::prelude::*;

use super::bytes::Bytes;
use super::task_api::{Promise, PromiseValue, TaskOutput};
use super::*;

const PROCESS_READ_CHUNK_SIZE: usize = 4096;

pub struct ProcessApi {
    lua: Lua,

    process_exec: LuaFunctionBuilder,
    process_open: LuaFunctionBuilder,
    process_stdin: LuaFunction,
    process_stdout: LuaFunction,
    process_stderr: LuaFunction,
    process_kill: LuaFunction,
    process_wait: LuaFunction,
    process_finished: LuaFunction
}

impl ProcessApi {
    pub fn new(lua: Lua) -> Result<Self, LuaError> {
        let process_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            process_exec: Box::new(|lua: &Lua, context: &Context| {
                let context = context.to_owned();
                let module_folder = context.module_folder.clone();

                lua.create_function(move |lua, (binary, args, env): (String, Option<LuaTable>, Option<LuaTable>)| {
                    let module_folder = module_folder.clone();

                    let value = PromiseValue::from_blocking(move || {
                        let mut command = Command::new(binary);

                        let mut command = command
                            .current_dir(&module_folder)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());

                        // Create module folder if it doesn't exist.
                        if !module_folder.is_dir() {
                            std::fs::create_dir_all(&module_folder)?;
                        }

                        // Apply command arguments.
                        if let Some(args) = args {
                            for arg in args.sequence_values::<LuaString>() {
                                command = command.arg(arg?.to_string_lossy());
                            }
                        }

                        // Apply command environment.
                        if let Some(env) = env {
                            for pair in env.pairs::<LuaString, LuaString>() {
                                let (key, value) = pair?;

                                command = command.env(
                                    key.to_string_lossy(),
                                    value.to_string_lossy()
                                );
                            }
                        }

                        #[cfg(feature = "tracing")]
                        tracing::debug!(?command, "running command");

                        // Execute the command.
                        let output = command.output()?;

                        Ok(Box::new(move |lua: &Lua| {
                            // Prepare the output.
                            let result = lua.create_table_with_capacity(0, 4)?;

                            let stdout = Bytes::new(output.stdout.into_boxed_slice());
                            let stderr = Bytes::new(output.stderr.into_boxed_slice());

                            result.raw_set("status", output.status.code())?;
                            result.raw_set("is_ok", output.status.success())?;
                            result.raw_set("stdout", stdout)?;
                            result.raw_set("stderr", stderr)?;

                            Ok(LuaValue::Table(result))
                        }) as TaskOutput)
                    });

                    Promise::new(value)
                        .into_lua(lua)
                })
            }),

            process_open: {
                let process_handles = process_handles.clone();

                Box::new(move |lua: &Lua, context: &Context| {
                    let context = context.to_owned();
                    let module_folder = context.module_folder.clone();
                    let process_handles = process_handles.clone();

                    lua.create_function(move |_, (binary, args, env): (String, Option<LuaTable>, Option<LuaTable>)| {
                        let mut command = Command::new(binary);

                        let mut command = command
                            .current_dir(&module_folder)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());

                        // Create module folder if it doesn't exist.
                        if !module_folder.is_dir() {
                            std::fs::create_dir_all(&module_folder)?;
                        }

                        // Apply command arguments.
                        if let Some(args) = args {
                            for arg in args.sequence_values::<LuaString>() {
                                command = command.arg(arg?.to_string_lossy());
                            }
                        }

                        // Apply command environment.
                        if let Some(env) = env {
                            for pair in env.pairs::<LuaString, LuaString>() {
                                let (key, value) = pair?;

                                command = command.env(
                                    key.to_string_lossy(),
                                    value.to_string_lossy()
                                );
                            }
                        }

                        // Start the process and store it.
                        let mut handles = process_handles.lock()
                            .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                        let mut handle = rand::random::<i32>();

                        while handles.contains_key(&handle) {
                            handle = rand::random::<i32>();
                        }

                        #[cfg(feature = "tracing")]
                        tracing::debug!(?command, "spawned process");

                        handles.insert(handle, command.spawn()?);

                        Ok(handle)
                    })
                })
            },

            process_stdin: {
                let process_handles = process_handles.clone();

                lua.create_function(move |_, (handle, data): (i32, Bytes)| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Try to write data to the process's stdin.
                    if let Some(stdin) = &mut process.stdin {
                        stdin.write_all(&data)?;
                    }

                    Ok(handle)
                })?
            },

            process_stdout: {
                let process_handles = process_handles.clone();

                lua.create_function(move |lua, handle: i32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Read the process's stdout chunk.
                    if let Some(stdout) = &mut process.stdout {
                        let mut buf = [0; PROCESS_READ_CHUNK_SIZE];

                        let len = stdout.read(&mut buf)?;

                        return Bytes::new(buf[..len].to_vec().into_boxed_slice())
                            .into_lua(lua);
                    }

                    Ok(LuaNil)
                })?
            },

            process_stderr: {
                let process_handles = process_handles.clone();

                lua.create_function(move |lua, handle: i32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Read the process's stderr chunk.
                    if let Some(stderr) = &mut process.stderr {
                        let mut buf = [0; PROCESS_READ_CHUNK_SIZE];

                        let len = stderr.read(&mut buf)?;

                        return Bytes::new(buf[..len].to_vec().into_boxed_slice())
                            .into_lua(lua);
                    }

                    Ok(LuaNil)
                })?
            },

            process_kill: {
                let process_handles = process_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Kill the process and remove its handle.
                    process.kill()?;
                    handles.remove(&handle);

                    Ok(())
                })?
            },

            process_wait: {
                let process_handles = process_handles.clone();

                lua.create_function(move |lua, handle: i32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.remove(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Wait until the process has finished.
                    let output = process.wait_with_output()?;

                    // Prepare lua result.
                    let result = lua.create_table_with_capacity(0, 4)?;

                    let stdout = Bytes::new(output.stdout.into_boxed_slice());
                    let stderr = Bytes::new(output.stderr.into_boxed_slice());

                    result.raw_set("status", output.status.code())?;
                    result.raw_set("is_ok", output.status.success())?;
                    result.raw_set("stdout", stdout)?;
                    result.raw_set("stderr", stderr)?;

                    Ok(result)
                })?
            },

            process_finished: {
                let process_handles = process_handles.clone();

                lua.create_function(move |_, handle: i32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    Ok(process.try_wait()?.is_some())
                })?
            },

            lua
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable, LuaError> {
        let env = self.lua.create_table_with_capacity(0, 8)?;

        env.raw_set("exec", (self.process_exec)(&self.lua, context)?)?;
        env.raw_set("open", (self.process_open)(&self.lua, context)?)?;
        env.raw_set("stdin", &self.process_stdin)?;
        env.raw_set("stdout", &self.process_stdout)?;
        env.raw_set("stderr", &self.process_stderr)?;
        env.raw_set("wait", &self.process_wait)?;
        env.raw_set("kill", &self.process_kill)?;
        env.raw_set("finished", &self.process_finished)?;

        Ok(env)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn process_exec() -> anyhow::Result<()> {
//         let api = ProcessApi::new(Lua::new())?;

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

//         let output = env.call_function::<LuaTable>("exec", (
//             "bash", ["-c", "echo $TEST"],
//             HashMap::from([
//                 ("TEST", "Hello, World!")
//             ])
//         ))?;

//         assert_eq!(output.get::<i32>("status")?, 0);
//         assert!(output.get::<bool>("is_ok")?);
//         assert_eq!(output.get::<Vec<u8>>("stdout")?, b"Hello, World!\n");

//         Ok(())
//     }

//     #[test]
//     fn process_open() -> anyhow::Result<()> {
//         let api = ProcessAPI::new(Lua::new())?;

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

//         let handle = env.call_function::<i32>("open", (
//             "bash", ["-c", "echo $TEST"],
//             HashMap::from([
//                 ("TEST", "Hello, World!")
//             ])
//         ))?;

//         while !api.process_finished.call::<bool>(handle)? {
//             std::thread::sleep(std::time::Duration::from_millis(100));
//         }

//         assert_eq!(api.process_stdout.call::<Vec<u8>>(handle)?, b"Hello, World!\n");

//         let output = api.process_wait.call::<LuaTable>(handle)?;

//         assert_eq!(output.get::<i32>("status")?, 0);
//         assert!(output.get::<bool>("is_ok")?);
//         assert!(output.get::<Vec<u8>>("stdout")?.is_empty());

//         Ok(())
//     }
// }
