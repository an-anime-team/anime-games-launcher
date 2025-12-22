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

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use mlua::prelude::*;

#[cfg(feature = "packages-support")]
use agl_packages::{
    hash::Hash,
    format::ResourceFormat,
    storage::Storage,
    lock::Lock
};

#[cfg(feature = "packages-support")]
use crate::allow_list::AllowList;

use crate::module::{Module, ModuleScope};
use crate::api::{Api, ApiOptions, Context};

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

    #[cfg(feature = "packages-support")]
    #[error("packages lock is missing a package with hash '{}'", hash.to_base32())]
    LockPackageMissing {
        hash: Hash
    },

    #[cfg(feature = "packages-support")]
    #[error("module with hash '{}' has duplicate input under name '{input_name}'", module_hash.to_base32())]
    ModuleHasDuplicateInput {
        module_hash: Hash,
        input_name: String
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModulePaths {
    /// Path to the folder where temporary files should be stored. This folder
    /// is accessible by all the modules and is expected that will be cleaned
    /// automatically eventually.
    pub temp_folder: PathBuf,

    /// Path to the folder where modules will have their own secure storages.
    /// Each module will get a subfolder which can be accessed only by this
    /// module.
    pub modules_folder: PathBuf,

    /// Path to the folder where persistent files should be stored. This folder
    /// is accessible by all the modules and is expected that it will not be
    /// cleaned in long amount of time, so that modules can store important
    /// files there (e.g. downloaded game files).
    pub persistent_folder: PathBuf
}

/// A host struct for luau scripts runtime. Allows to spawn new scripts and
/// provide them with scoped permissions.
pub struct Runtime {
    lua: Lua,
    api: Api
}

impl Runtime {
    /// Create new luau engine with provided API options.
    pub fn new(options: ApiOptions) -> Result<Self, RuntimeError> {
        // Prepare tables and create a registry key to be able to access them
        // from the rust side.
        let engine_table = options.lua.create_table_with_capacity(0, 2)?;

        let values_table = options.lua.create_table()?;
        let refs_table = options.lua.create_table()?;

        engine_table.raw_set("values", values_table.clone())?; // [value_key] => [value]
        engine_table.raw_set("refs", refs_table.clone())?;     // [value_key] => { [name] => [value_key] }

        options.lua.set_named_registry_value("engine", engine_table)?;

        // Enable JIT and sandbox for modules execution.
        options.lua.enable_jit(true);
        options.lua.sandbox(true)?;

        Ok(Self {
            lua: options.lua.clone(),
            api: Api::new(options)?
        })
    }

    #[inline(always)]
    pub const fn lua(&self) -> &Lua {
        &self.lua
    }

    /// Try to create a luau module environment from provided permissions scope.
    fn create_env_from_scope(
        &self,
        temp_folder: PathBuf,
        module_folder: PathBuf,
        persistent_folder: PathBuf,
        module_key: String,
        scope: ModuleScope
    ) -> Result<LuaTable, RuntimeError> {
        // Create environment table with the standard library APIs.
        let env = self.api.create_env(&Context {
            temp_folder,
            module_folder,
            persistent_folder,

            scope: Arc::new(RwLock::new(scope))
        })?;

        // Load referenced value.
        env.set("load", self.lua.create_function(move |lua, name: String| -> Result<LuaValue, LuaError> {
            // Read the engine table from the registry key.
            let engine_table = lua.named_registry_value::<LuaTable>("engine")?;

            // Read the values and refs tables from the engine.
            let values_table = engine_table.raw_get::<LuaTable>("values")?;
            let refs_table = engine_table.raw_get::<LuaTable>("refs")?;

            // If current module doesn't reference any values - return `nil`.
            if !refs_table.contains_key(module_key.as_str())? {
                return Ok(LuaValue::Nil);
            }

            // Read the module's references.
            let module_refs_table = refs_table.raw_get::<LuaTable>(module_key.as_str())?;

            // If current module doesn't have reference with provided name -
            // return `nil`.
            if !module_refs_table.contains_key(name.as_str())? {
                return Ok(LuaValue::Nil);
            }

            // Read the referenced value's key.
            let ref_key = module_refs_table.raw_get::<String>(name)?;

            // Read the referenced value.
            values_table.raw_get(ref_key)
        })?)?;

        Ok(env)
    }

    /// Try to set luau value to the runtime key-value storage.
    pub fn set_value(
        &self,
        key: impl AsRef<str>,
        value: impl IntoLua
    ) -> Result<(), LuaError> {
        // Load value using the luau engine.
        let value = value.into_lua(&self.lua)?;

        // Read the engine table from the registry key.
        let engine_table = self.lua.named_registry_value::<LuaTable>("engine")?;

        // Read the values table from the engine.
        let values_table = engine_table.raw_get::<LuaTable>("values")?;

        // Store the value.
        values_table.raw_set(key.as_ref(), value)?;

        Ok(())
    }

    /// Try to get luau value from the runtime key-value storage.
    pub fn get_value<T: FromLua>(
        &self,
        key: impl AsRef<str>
    ) -> Result<Option<T>, LuaError> {
        // Read the engine table from the registry key.
        let engine_table = self.lua.named_registry_value::<LuaTable>("engine")?;

        // Read the values table from the engine.
        let values_table = engine_table.raw_get::<LuaTable>("values")?;

        // Return `None` if there's no value with provided key.
        if !values_table.contains_key(key.as_ref())? {
            return Ok(None);
        }

        // Read the value.
        values_table.raw_get(key.as_ref()).map(Some)
    }

    /// Reference `target_key` value in a value (module) with key `source_key`
    /// using `name` as a reference name.
    pub fn set_named_reference(
        &self,
        source_key: impl AsRef<str>,
        target_key: impl AsRef<str>,
        name: impl AsRef<str>
    ) -> Result<(), RuntimeError> {
        // Read the engine table from the registry key.
        let engine_table = self.lua.named_registry_value::<LuaTable>("engine")?;

        // Read the references table from the engine.
        let refs_table = engine_table.raw_get::<LuaTable>("refs")?;

        // Insert new empty refs table if it doesn't exist.
        if !refs_table.contains_key(source_key.as_ref())? {
            let value_refs_table = self.lua.create_table_with_capacity(0, 1)?;

            refs_table.raw_set(source_key.as_ref(), value_refs_table)?;
        }

        // Read the value's refs table.
        let value_refs_table = refs_table.raw_get::<LuaTable>(source_key.as_ref())?;

        // Insert the named reference.
        value_refs_table.raw_set(name.as_ref(), target_key.as_ref())?;

        Ok(())
    }

    /// Try to load new luau module into the runtime. The module's output will
    /// be stored in the runtime key-value storage under provided key.
    pub fn load_module(
        &self,
        key: impl ToString,
        module: Module,
        paths: ModulePaths
    ) -> Result<(), RuntimeError> {
        // Check if the module file exists and is a readable file.
        if !module.path.is_file() {
            return Err(RuntimeError::ModuleDoesntExist(module.path));
        }

        // Read the module file.
        let module_content = std::fs::read(&module.path)
            .map_err(|err| {
                RuntimeError::ModuleReadError {
                    path: module.path.clone(),
                    err
                }
            })?;

        let module_hash = Hash::from_bytes(&module_content);

        // Read the engine table from the registry key.
        let engine_table = self.lua.named_registry_value::<LuaTable>("engine")?;

        // Read the values table from the engine.
        let values_table = engine_table.raw_get::<LuaTable>("values")?;

        // Get the module key.
        let key = key.to_string();

        // Create environment for the module.
        let env = self.create_env_from_scope(
            paths.temp_folder,
            paths.modules_folder.join(module_hash.to_base32()),
            paths.persistent_folder,
            key.clone(),
            module.scope
        )?;

        // Execute the module.
        let result = self.lua.load(module_content)
            .set_name(module.path.to_string_lossy())
            .set_environment(env)
            .call::<LuaValue>(())?;

        // Insert the module's result into the table.
        values_table.raw_set(key, result)?;

        Ok(())
    }

    /// Try to load all the packages and luau modules from provided Anime Games
    /// Launcher packages manager lock.
    #[cfg(feature = "packages-support")]
    pub fn load_packages(
        &self,
        lock: &Lock,
        storage: &Storage,
        paths: &ModulePaths,
        allow_list: &AllowList
    ) -> Result<(), RuntimeError> {
        use std::collections::{VecDeque, HashSet, HashMap};

        // TODO: implement something like RichResourceFormat with Module format
        //       instead of doing shit with is_module_resource and so

        #[inline]
        fn get_resource_key(
            hash: impl std::fmt::Display,
            format: impl std::fmt::Display
        ) -> String {
            format!("{hash}#{format}")
        }

        #[inline]
        fn is_module_resource(url: &str) -> bool {
            url.ends_with(".lua") || url.ends_with(".luau")
        }

        // Prepare the packages queue and set of processed packages.
        let mut packages_queue = VecDeque::with_capacity(lock.packages.len());
        let mut processed_packages = HashSet::new();

        // Prepare the [module_hash] => { [input_name] => [resource_key] } table.
        let mut modules_table = HashMap::new();

        // Enqueue root packages.
        packages_queue.extend(lock.root.iter().copied());

        // Iterate over all the queued packages.
        while let Some(hash) = packages_queue.pop_front() {
            // Skip already processed package.
            if processed_packages.contains(&hash) {
                continue;
            }

            // Try to read the package's info or return an error if it's
            // missing.
            let Some(package) = lock.packages.get(&hash) else {
                return Err(RuntimeError::LockPackageMissing {
                    hash
                });
            };

            // Iterate over the package's resources.
            let resources = package.inputs.values()
                .chain(package.outputs.values());

            for resource in resources {
                let resource_key = if is_module_resource(&resource.url) {
                    get_resource_key(resource.hash.to_base32(), "module")
                } else {
                    get_resource_key(resource.hash.to_base32(), resource.format.to_string())
                };

                match resource.format {
                    // Order package's processing.
                    ResourceFormat::Package => {
                        packages_queue.push_back(resource.hash);
                    }

                    // Schedule lua or luau files loading as modules.
                    ResourceFormat::File if is_module_resource(&resource.url) => {
                        let module: &mut HashMap<String, String> = modules_table.entry(resource.hash)
                            .or_default();

                        for (name, input_resource) in package.inputs.iter() {
                            let input_resource_key = if is_module_resource(&input_resource.url) {
                                get_resource_key(input_resource.hash.to_base32(), "module")
                            } else {
                                get_resource_key(input_resource.hash.to_base32(), input_resource.format.to_string())
                            };

                            // Prevent names re-assigning.
                            if let Some(duplicate_key) = module.get(name)
                                && duplicate_key != &input_resource_key
                            {
                                return Err(RuntimeError::ModuleHasDuplicateInput {
                                    module_hash: input_resource.hash,
                                    input_name: name.to_string()
                                });
                            }

                            module.insert(name.to_string(), input_resource_key);
                        }
                    }

                    // Load normal files or archives as filesystem path values.
                    ResourceFormat::File | ResourceFormat::Archive => {
                        let value = self.lua.create_table_with_capacity(0, 3)?;

                        value.raw_set("hash", resource.hash.to_base32())?;
                        value.raw_set("format", resource.format.to_string())?;
                        value.raw_set("value", storage.resource_path(&resource.hash))?;

                        self.set_value(resource_key, value)?;
                    }
                }
            }

            // Add package as runtime value.

            // Prepare package outputs table.
            // We don't need to store package inputs since they're private.
            let package_value = self.lua.create_table_with_capacity(0, package.outputs.len())?;

            // Insert output values.
            for (name, resource) in package.outputs.iter() {
                let value_key = if is_module_resource(&resource.url) {
                    get_resource_key(resource.hash.to_base32(), "module")
                } else {
                    get_resource_key(resource.hash.to_base32(), resource.format.to_string())
                };

                package_value.raw_set(
                    name.to_string(),
                    value_key
                )?;
            }

            // Prepare runtime value table.
            let value = self.lua.create_table_with_capacity(0, 3)?;

            value.raw_set("hash", hash.to_base32())?;
            value.raw_set("format", ResourceFormat::Package.to_string())?;
            value.raw_set("value", package_value)?;

            // Store the value table.
            let package_key = get_resource_key(
                hash.to_base32(),
                ResourceFormat::Package.to_string()
            );

            self.set_value(package_key, value)?;

            // Mark current package as processed.
            processed_packages.insert(hash);
        }

        // Load modules.
        for (hash, inputs) in modules_table.iter() {
            let module_key = get_resource_key(hash.to_base32(), "module");

            let mut module = Module {
                path: storage.resource_path(hash),
                scope: allow_list.get_module_scope(hash)
                    .cloned()
                    .unwrap_or_default()
            };

            // Add package inputs as allowed paths to the output module of the
            // same package.
            for resource_key in inputs.values() {
                // TODO: avoid reading lua values here...
                if let Ok(Some(resource_value)) = self.get_value::<LuaTable>(resource_key) {
                    let format = resource_value.raw_get::<String>("format")?;

                    if format == "file" || format == "archive" {
                        let resource_path = resource_value.raw_get::<PathBuf>("value")?;

                        module.scope.sandbox_read_paths.push(resource_path);
                    }
                }
            }

            self.load_module(module_key.clone(), module, paths.clone())?;

            // Read the loaded module's result.
            let module_result = self.get_value::<LuaValue>(&module_key)?;

            // Prepare formatted table.
            let result_table = self.lua.create_table_with_capacity(0, 3)?;

            result_table.raw_set("hash", hash.to_base32())?;
            result_table.raw_set("format", "module")?;
            result_table.raw_set("value", module_result)?;

            // Replace the module's result with formatted table.
            self.set_value(module_key, result_table)?;
        }

        // Add input references to all the loaded modules.
        //
        // We're doing it *after* loading all the modules since one module
        // could reference the other one, which could happen to not be
        // loaded yet.
        for (hash, inputs) in modules_table {
            let module_key = get_resource_key(hash, "module");

            // Iterate over the module's inputs.
            for (name, input_key) in inputs {
                // Add named reference to the input.
                self.set_named_reference(&module_key, input_key, name)?;
            }
        }

        // Update packages outputs.
        for hash in processed_packages {
            let package_key = get_resource_key(
                hash.to_base32(),
                ResourceFormat::Package.to_string()
            );

            if let Some(package_table) = self.get_value::<LuaTable>(&package_key)? {
                let package_values = package_table.raw_get::<LuaTable>("value")?;

                // Iterate over the package's outputs.
                for result in package_values.pairs::<String, String>() {
                    let (output_name, output_value_key) = result?;

                    // Replace key by its stored value.
                    package_values.raw_set(
                        output_name,
                        self.get_value::<LuaValue>(output_value_key)?
                    )?;
                }
            }
        }

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
