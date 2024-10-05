use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use std::process::{Command, Stdio};

use mlua::prelude::*;

use super::*;

const PROCESS_READ_CHUNK_LEN: usize = 1024;

pub struct ProcessAPI<'lua> {
    lua: &'lua Lua,

    process_exec: LuaFunctionBuilder<'lua>,
    process_open: LuaFunctionBuilder<'lua>,
    process_stdin: LuaFunction<'lua>,
    process_stdout: LuaFunction<'lua>,
    process_stderr: LuaFunction<'lua>,
    process_kill: LuaFunction<'lua>,
    process_wait: LuaFunction<'lua>,
    process_finished: LuaFunction<'lua>
}

impl<'lua> ProcessAPI<'lua> {
    pub fn new(lua: &'lua Lua) -> Result<Self, EngineError> {
        let process_handles = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            lua,

            process_exec: Box::new(|lua: &'lua Lua, context: &Context| {
                let module_folder = context.module_folder.clone();

                lua.create_function(move |lua, (path, args, env): (LuaString, Option<LuaTable>, Option<LuaTable>)| {
                    let path = resolve_path(path.to_string_lossy())?;

                    let mut command = Command::new(path);

                    let mut command = command
                        .current_dir(&module_folder)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());

                    // Apply command arguments.
                    if let Some(args) = args {
                        for arg in args.sequence_values::<LuaString>() {
                            command = command.arg(arg?.to_string_lossy().to_string());
                        }
                    }

                    // Apply command environment.
                    if let Some(env) = env {
                        for pair in env.pairs::<LuaString, LuaString>() {
                            let (key, value) = pair?;

                            command = command.env(
                                key.to_string_lossy().to_string(),
                                value.to_string_lossy().to_string()
                            );
                        }
                    }

                    // Execute the command.
                    let output = command.output()?;

                    // Prepare the output.
                    let result = lua.create_table()?;

                    result.set("status", output.status.code())?;
                    result.set("is_ok", output.status.success())?;
                    result.set("stdout", output.stdout)?;
                    result.set("stderr", output.stderr)?;

                    Ok(result)
                })
            }),

            process_open: {
                let process_handles = process_handles.clone();

                Box::new(move |lua: &'lua Lua, context: &Context| {
                    let module_folder = context.module_folder.clone();
                    let process_handles = process_handles.clone();

                    lua.create_function(move |_, (path, args, env): (LuaString, Option<LuaTable>, Option<LuaTable>)| {
                        let path = resolve_path(path.to_string_lossy())?;

                        let mut command = Command::new(path);

                        let mut command = command
                            .current_dir(&module_folder)
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped());

                        // Apply command arguments.
                        if let Some(args) = args {
                            for arg in args.sequence_values::<LuaString>() {
                                command = command.arg(arg?.to_string_lossy().to_string());
                            }
                        }

                        // Apply command environment.
                        if let Some(env) = env {
                            for pair in env.pairs::<LuaString, LuaString>() {
                                let (key, value) = pair?;

                                command = command.env(
                                    key.to_string_lossy().to_string(),
                                    value.to_string_lossy().to_string()
                                );
                            }
                        }

                        // Start the process and store it.
                        let mut handles = process_handles.lock()
                            .map_err(|err| LuaError::external(format!("failed to register handle: {err}")))?;

                        let mut handle = rand::random::<u32>();

                        while handles.contains_key(&handle) {
                            handle = rand::random::<u32>();
                        }

                        handles.insert(handle, command.spawn()?);

                        Ok(handle)
                    })
                })
            },

            process_stdin: {
                let process_handles = process_handles.clone();

                lua.create_function(move |_, (handle, data): (u32, LuaValue)| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Try to write data to the process's stdin.
                    if let Some(stdin) = &mut process.stdin {
                        stdin.write_all(&get_value_bytes(data)?)?;
                    }

                    Ok(handle)
                })?
            },

            process_stdout: {
                let process_handles = process_handles.clone();

                lua.create_function(move |lua, handle: u32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Read the process's stdout chunk.
                    if let Some(stdout) = &mut process.stdout {
                        let mut buf = [0; PROCESS_READ_CHUNK_LEN];

                        let len = stdout.read(&mut buf)?;

                        return slice_to_table(lua, &buf[..len])
                            .map(LuaValue::Table);
                    }

                    Ok(LuaNil)
                })?
            },

            process_stderr: {
                let process_handles = process_handles.clone();

                lua.create_function(move |lua, handle: u32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Read the process's stderr chunk.
                    if let Some(stderr) = &mut process.stderr {
                        let mut buf = [0; PROCESS_READ_CHUNK_LEN];

                        let len = stderr.read(&mut buf)?;

                        return slice_to_table(lua, &buf[..len])
                            .map(LuaValue::Table);
                    }

                    Ok(LuaNil)
                })?
            },

            process_kill: {
                let process_handles = process_handles.clone();

                lua.create_function(move |_, handle: u32| {
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

                lua.create_function(move |lua, handle: u32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.remove(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    // Wait until the process has finished.
                    let output = process.wait_with_output()?;

                    // Prepare lua result.
                    let result = lua.create_table()?;

                    result.set("status", output.status.code())?;
                    result.set("is_ok", output.status.success())?;
                    result.set("stdout", output.stdout)?;
                    result.set("stderr", output.stderr)?;

                    Ok(result)
                })?
            },

            process_finished: {
                let process_handles = process_handles.clone();

                lua.create_function(move |_, handle: u32| {
                    let mut handles = process_handles.lock()
                        .map_err(|err| LuaError::external(format!("failed to read handle: {err}")))?;

                    let Some(process) = handles.get_mut(&handle) else {
                        return Err(LuaError::external("invalid process handle"));
                    };

                    Ok(process.try_wait()?.is_some())
                })?
            }
        })
    }

    /// Create new lua table with API functions.
    pub fn create_env(&self, context: &Context) -> Result<LuaTable<'lua>, EngineError> {
        let env = self.lua.create_table_with_capacity(0, 8)?;

        env.set("exec", (self.process_exec)(self.lua, context)?)?;
        env.set("open", (self.process_open)(self.lua, context)?)?;
        env.set("stdin", self.process_stdin.clone())?;
        env.set("stdout", self.process_stdout.clone())?;
        env.set("stderr", self.process_stderr.clone())?;
        env.set("wait", self.process_wait.clone())?;
        env.set("kill", self.process_kill.clone())?;
        env.set("finished", self.process_finished.clone())?;

        Ok(env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_exec() -> anyhow::Result<()> {
        let lua = Lua::new();
        let api = ProcessAPI::new(&lua)?;

        let env = api.create_env(&Context {
            temp_folder: std::env::temp_dir(),
            module_folder: std::env::temp_dir(),
            persistent_folder: std::env::temp_dir(),
            ext_process_api: false
        })?;

        let output = env.call_function::<_, LuaTable>("exec", (
            "bash", ["-c", "echo $TEST"],
            HashMap::from([
                ("TEST", "Hello, World!")
            ])
        ))?;

        assert_eq!(output.get::<_, i32>("status")?, 0);
        assert!(output.get::<_, bool>("is_ok")?);
        assert_eq!(output.get::<_, Vec<u8>>("stdout")?, b"Hello, World!\n");

        Ok(())
    }

    #[test]
    fn process_open() -> anyhow::Result<()> {
        let lua = Lua::new();
        let api = ProcessAPI::new(&lua)?;

        let env = api.create_env(&Context {
            temp_folder: std::env::temp_dir(),
            module_folder: std::env::temp_dir(),
            persistent_folder: std::env::temp_dir(),
            ext_process_api: false
        })?;

        let handle = env.call_function::<_, u32>("open", (
            "bash", ["-c", "echo $TEST"],
            HashMap::from([
                ("TEST", "Hello, World!")
            ])
        ))?;

        while !api.process_finished.call::<_, bool>(handle)? {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        assert_eq!(api.process_stdout.call::<_, Vec<u8>>(handle)?, b"Hello, World!\n");

        let output = api.process_wait.call::<_, LuaTable>(handle)?;

        assert_eq!(output.get::<_, i32>("status")?, 0);
        assert!(output.get::<_, bool>("is_ok")?);
        assert!(output.get::<_, Vec<u8>>("stdout")?.is_empty());

        Ok(())
    }
}
