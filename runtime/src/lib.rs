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

/// Version of the `agl-runtime` library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod api;

use std::path::PathBuf;

use mlua::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("luau engine error: {0}")]
    Luau(#[from] LuaError),

    #[error("luau module file with provided path does not exist: {0:?}")]
    ModuleDoesntExist(PathBuf),

    #[error("failed to read luau module file at path '{path:?}': {err}")]
    ModuleReadError {
        path: PathBuf,
        err: std::io::Error
    },

    #[error("module with hash '{hash}' at path '{path:?}' is already loaded")]
    ModuleAlreadyLoaded {
        hash: u64,
        path: PathBuf
    },

    #[error("module with hash '{module_hash}' already has a resource with name '{resource_name}'")]
    ModuleInputAlreadyExists {
        module_hash: u64,
        resource_name: String
    }
}

/// A host struct for luau scripts runtime. Allows to spawn new scripts and
/// provide them with scoped permissions.
pub struct Runtime {
    lua: Lua,
    api: api::Api
}

impl Runtime {
    /// Create new luau engine.
    pub fn new() -> Result<Self, RuntimeError> {
        // Create luau engine.
        let lua = Lua::new();

        // Prepare tables and create a registry key to be able to access them
        // from the rust side.
        let engine_table = lua.create_table_with_capacity(0, 3)?;

        let modules_table = lua.create_table()?;
        let resources_table = lua.create_table()?;
        let inputs_table = lua.create_table()?;

        engine_table.raw_set("modules", modules_table.clone())?;     //   [module_hash] => [module_output]
        engine_table.raw_set("resources", resources_table.clone())?; // [resource_hash] => [resource_value]
        engine_table.raw_set("inputs", inputs_table.clone())?;       //   [module_hash] => { [resource_name] => [resource_hash] }

        lua.set_named_registry_value("engine", engine_table)?;

        // Enable sandbox for modules execution.
        lua.sandbox(true)?;

        Ok(Self {
            lua: lua.clone(),
            api: api::Api::new(lua)?
        })
    }

    /// Try to create a luau module environment from provided permissions scope.
    fn create_env_from_scope(
        &self,
        hash: u64,
        scope: ModuleScope
    ) -> Result<LuaTable, RuntimeError> {
        // Create environment table with the standard library APIs.
        let env = self.api.create_env(&api::Context {
            module_hash: hash,

            // TODO
            temp_folder: std::env::temp_dir(),
            module_folder: std::env::temp_dir().join(hash.to_string()),
            persistent_folder: std::env::temp_dir(),

            scope
        })?;

        // Load dependency module or resource.
        env.set("load", self.lua.create_function(move |lua, name: String| -> Result<LuaTable, LuaError> {
            // Read the engine table from the registry key.
            let engine_table = lua.named_registry_value::<LuaTable>("engine")?;

            // Read the inputs table from the engine.
            let inputs_table = engine_table.raw_get::<LuaTable>("inputs")?;

            // Try to read current module input resources.
            let Ok(module_inputs) = inputs_table.raw_get::<LuaTable>(hash.to_string()) else {
                return Err(LuaError::external("module '{hash}' doesn't have any inputs"));
            };

            // Try to read input resource hash from its name.
            let Ok(resource_hash) = module_inputs.raw_get::<String>(name.as_str()) else {
                return Err(LuaError::external(format!("module '{hash}' missing dependency called '{name}'")));
            };

            // Read the resources table from the engine.
            let resources_table = engine_table.raw_get::<LuaTable>("resources")?;

            // Try to read resource value from its hash.
            let Ok(resource) = resources_table.raw_get::<LuaValue>(resource_hash.as_str()) else {
                return Err(LuaError::external(format!("missing resource with hash '{resource_hash}'")));
            };

            // TODO: support importing sub-modules (right now only resources can be imported).

            // Prepare function output.
            let output_table = lua.create_table_with_capacity(0, 3)?;

            output_table.raw_set("hash", resource_hash.as_str())?;
            output_table.raw_set("format", "resource")?;
            output_table.raw_set("value", resource)?;

            Ok(output_table)
        })?)?;

        // TODO: implement `import` function.

        Ok(env)
    }

    /// Try to load new luau module into the runtime.
    ///
    /// - If the module with provided hash is already loaded, then its scope
    ///   will be updated to allow new permissions if some were diallowed.
    /// - If the module was not loaded, then it will be attempted to load.
    ///
    /// Hash of the module will be returned. It can be used to add dependencies
    /// to it.
    ///
    /// > **Important note:** due to the runtime nature it's impossible to load
    /// > the same module multiple times. Attempts to load the same module will
    /// > lead to a runtime error. You must merge all the repeating modules and
    /// > their scopes before using this method.
    pub fn load_module(
        &mut self,
        module: Module
    ) -> Result<u64, RuntimeError> {
        // Check if the module file exists and is a readable file.
        if !module.path.is_file() {
            return Err(RuntimeError::ModuleDoesntExist(module.path));
        }

        // Read the module file.
        let file = std::fs::read(&module.path)
            .map_err(|err| {
                RuntimeError::ModuleReadError {
                    path: module.path.clone(),
                    err
                }
            })?;

        // Calculate module hash.
        let hash = seahash::hash(&file);

        // Read the engine table from the registry key.
        let engine_table = self.lua.named_registry_value::<LuaTable>("engine")?;

        // Read the modules table from the engine.
        let modules_table = engine_table.raw_get::<LuaTable>("modules")?;

        // Prevent module double-loading.
        //
        // We can't merge scope permissions because it would require module to
        // be re-loaded with new environment which is not intended behavior.
        if modules_table.contains_key(hash.to_string())? {
            return Err(RuntimeError::ModuleAlreadyLoaded {
                hash,
                path: module.path
            });
        }

        // Create environment for the module.
        let env = self.create_env_from_scope(hash, module.scope)?;

        // Execute the module.
        let result = self.lua.load(file)
            .set_name(module.path.to_string_lossy())
            .set_environment(env)
            .call::<LuaValue>(())?;

        // Insert the module's result into the table.
        modules_table.raw_set(hash.to_string(), result)?;

        Ok(hash)
    }

    // TODO: also add something like module_add_dependency/submodule

    /// Try to add a resource value into the loaded module with provided hash.
    pub fn module_add_input_resource(
        &self,
        hash: u64,
        name: impl AsRef<str>,
        value: impl IntoLua
    ) -> Result<(), RuntimeError> {
        // Obtain the resource value.
        let value = value.into_lua(&self.lua)?;

        // Cast this value to a string and calculate its hash.
        // Technically a huge performance loss....
        let resource_hash = seahash::hash(value.to_string()?.as_bytes());

        // Read the engine table from the registry key.
        let engine_table = self.lua.named_registry_value::<LuaTable>("engine")?;

        // Read the resources and inputs tables from the engine.
        let resources_table = engine_table.raw_get::<LuaTable>("resources")?;
        let inputs_table = engine_table.raw_get::<LuaTable>("inputs")?;

        // Read module inputs table.
        let module_inputs_table = match inputs_table.raw_get::<LuaTable>(hash.to_string()) {
            Ok(module_inputs_table) => module_inputs_table,

            Err(_) => {
                let module_inputs_table = self.lua.create_table_with_capacity(0, 1)?;

                inputs_table.raw_set(
                    hash.to_string(),
                    module_inputs_table.clone()
                )?;

                module_inputs_table
            }
        };

        // Check if module already has an input with this name.
        if module_inputs_table.contains_key(name.as_ref())? {
            return Err(RuntimeError::ModuleInputAlreadyExists {
                module_hash: hash,
                resource_name: name.as_ref().to_string()
            });
        }

        // Reference the input in the module.
        resources_table.raw_set(resource_hash.to_string(), value)?;
        module_inputs_table.raw_set(name.as_ref(), resource_hash.to_string())?;

        Ok(())
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        let _ = self.lua.unset_named_registry_value("engine");

        self.lua.expire_registry_values();

        let _ = self.lua.gc_collect();
        let _ = self.lua.gc_collect();
    }
}

/// A luau module description.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Module {
    /// Path to the luau module file.
    pub path: PathBuf,

    /// Luau module permissions.
    pub scope: ModuleScope
}

/// Luau module permissions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModuleScope {
    /// Allow module to access String API.
    ///
    /// This API allows module to perform conversions between different string
    /// encodings (UTF-8, ASCII, etc.) and formats (hex, base64, JSON, etc.).
    ///
    /// Default: `true`.
    pub allow_string_api: bool,

    /// Allow module to access Path API.
    ///
    /// This API allows module to combine different path parts, normalize and
    /// resolve them, check if files or folders exist and if they're accessible.
    ///
    /// Default: `true`.
    pub allow_path_api: bool,

    /// Allow module to access basic Filesystem API.
    ///
    /// This API allows module to perform read/write/create operations on files
    /// and folders of the host filesystem, with sandboxed access to only
    /// allowed files and folders.
    ///
    /// Default: `true`.
    pub allow_basic_fs_api: bool,

    /// Allow module to access network API.
    ///
    /// This API allows module to perform HTTP requests.
    ///
    /// Default: `true`.
    pub allow_network_api: bool,

    /// Allow module to access Downloader API.
    ///
    /// This API allows module to download files from HTTP servers. Similar to
    /// the Network API, except it has more user niceness in it.
    ///
    /// Default: `true`.
    pub allow_downloader_api: bool,

    /// Allow module to access Archives API.
    ///
    /// This API allows module to extract archives or list their info.
    ///
    /// Default: `true`.
    pub allow_archives_api: bool,

    /// Allow module to access Hashes API.
    ///
    /// This API allows module to calculate different hashes of files or
    /// folders.
    ///
    /// Default: `true`.
    pub allow_hashes_api: bool,

    /// Allow module to access Compression API.
    ///
    /// This API allows module to compress or decompress data with different
    /// compression algorithms.
    ///
    /// Default: `true`.
    pub allow_compression_api: bool,

    /// Allow module to access Sqlite API.
    ///
    /// This API allows module to work with a sqlite database.
    ///
    /// Default: `true`.
    pub allow_sqlite_api: bool,

    /// Allow module to access Process API.
    ///
    /// This API allows module to spawn and control new processes on the host
    /// system.
    ///
    /// > **Security warning:** This API can be used to escape the sandbox. You
    /// > must make sure that the module *really* needs this API.
    ///
    /// Default: `false`.
    pub allow_process_api: bool,

    /// Paths allowed to be accessed for this module. When provided, the module
    /// can use Filesystem and other APIs to read or write into provided files
    /// or folders/subfolders.
    ///
    /// Default: none.
    pub sandbox_allowed_paths: Vec<PathBuf>
}

impl Default for ModuleScope {
    fn default() -> Self {
        Self {
            allow_string_api: true,
            allow_path_api: true,
            allow_basic_fs_api: true,
            allow_network_api: true,
            allow_downloader_api: true,
            allow_archives_api: true,
            allow_hashes_api: true,
            allow_compression_api: true,
            allow_sqlite_api: true,
            allow_process_api: false,
            sandbox_allowed_paths: vec![]
        }
    }
}
