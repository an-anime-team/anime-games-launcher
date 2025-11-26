use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use std::str::FromStr;

use mlua::prelude::*;

use crate::hash::Hash;
use crate::packages::lock_file::LockFile;
use crate::packages::store::ResourceStore;
use crate::packages::manifest::{ResourceFormat, ResourceModuleFormat};

pub mod api;

#[derive(Debug, thiserror::Error)]
pub enum PackagesEngineError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Lua engine error: {0}")]
    Lua(#[from] LuaError),

    #[error("Failed to lock lua registry key")]
    LuaRegistryKeyLock,

    #[error("Invalid resource format: {0}")]
    InvalidResourceFormat(String)
}

/// Packages engine options.
pub struct Options {
    /// Path to the temporary storage folder. It will be used by runtime modules
    /// to store temporary files which will be automatically wiped out by either
    /// operating system or some other logic.
    pub temp_store_path: PathBuf,

    /// Path to the persistent storage folder. It will be used by runtime
    /// modules to store files in long manner, for example, game files.
    pub persist_store_path: PathBuf,

    /// Path to the modules storage folder. It will be used by runtime modules
    /// to store their own, private information, which couldn't be read by other
    /// modules. It should act like persistent storage in terms of data storage
    /// duration.
    pub modules_store_path: PathBuf
}

// pub struct PackagesEngineOptions {
//     pub show_toast: Box<dyn Fn(v1_standard::ToastOptions) + Send>,
//     pub show_notification: Box<dyn Fn(v1_standard::NotificationOptions) + Send>,
//     pub show_dialog: Box<dyn Fn(v1_standard::DialogOptions) -> Option<String> + Send>
// }

pub struct PackagesEngine {
    lua: Lua,
    engine_registry: Arc<RwLock<LuaRegistryKey>>,
    lock_file: LockFile,
    _api: api::API
}

impl PackagesEngine {
    /// Create new packages engine and load all the resources from the provided
    /// lock file with permissions granted in the provided authority indexes.
    pub fn create(
        lua: Lua,
        store: &ResourceStore,
        lock_file: LockFile,
        // validator: AuthorityValidator,
        // local_validator: LocalValidator,
        options: Options
    ) -> Result<Self, PackagesEngineError> {
        let engine_table = lua.create_table()?;
        let resources_table = lua.create_table()?;

        // Lua tables are shared (like in JS) so I can store them right there.
        engine_table.set("resources", resources_table.clone())?;

        let engine_registry = Arc::new(RwLock::new(lua.create_registry_value(engine_table)?));

        let mut resources = Vec::with_capacity(lock_file.resources.len());
        let mut visited_resources = HashSet::new();
        let mut evaluation_queue = Vec::with_capacity(lock_file.resources.len());

        // Prepare standard folders. Clone paths here to reuse them later.
        let temp_store_path = options.temp_store_path.clone();
        let persist_store_path = options.persist_store_path.clone();
        let modules_store_path = options.modules_store_path.clone();

        if !persist_store_path.exists() {
            std::fs::create_dir_all(&persist_store_path)?;
        }

        if !temp_store_path.exists() {
            std::fs::create_dir_all(&temp_store_path)?;
        }

        // Prepare modules API implementations.
        let api = api::API::new(lua.clone(), options)?;

        // Push root resources to the processing queue.
        for root in &lock_file.lock.root {
            resources.push((*root, None));
        }

        // Iterate over all the locked resources.
        while let Some((key, parent_context)) = resources.pop() {
            // Skip if the resource was already processed.
            if !visited_resources.insert(key) {
                continue;
            }

            // Resolve base resource info.
            let resource = &lock_file.resources[key as usize];

            let path = store.get_path(&resource.lock.hash);

            // Store basic info to the lua representation.
            let resource_table = lua.create_table_with_capacity(0, 3)?;

            resource_table.set("format", resource.format.to_string())?;
            resource_table.set("hash", resource.lock.hash.to_base32())?;

            resources_table.set(key, resource_table.clone())?;

            // Prepare resource value depending on its format.
            match resource.format {
                ResourceFormat::Package => {
                    let package = lua.create_table()?;

                    let inputs_table = lua.create_table()?;
                    let outputs_table = lua.create_table()?;

                    package.set("inputs", inputs_table.clone())?;
                    package.set("outputs", outputs_table.clone())?;

                    resource_table.set("value", package.clone())?;

                    // Load inputs and outputs of the package.
                    // Push everything, regardless of whether
                    // the resource was already processed,
                    // to ensure correct loading order.
                    if let Some(outputs) = &resource.outputs {
                        for (name, output_key) in outputs.clone() {
                            outputs_table.set(name, output_key)?;

                            // Reference this package so the output module
                            // can load inputs of this package.
                            resources.push((output_key, Some(key)));
                        }
                    }

                    if let Some(inputs) = &resource.inputs {
                        for (name, input_key) in inputs.clone() {
                            inputs_table.set(name, input_key)?;

                            // Do not reference this package for inputs
                            // because inputs can't load other inputs.
                            resources.push((input_key, None));
                        }
                    }
                }

                ResourceFormat::Module(standard) => {
                    let module = std::fs::read(&path)?;
                    let module = lua.load(module);

                    let mut input_resources = vec![path.clone()];
                    let mut parent_hash = None;

                    if let Some(parent_context) = parent_context {
                        let engine_registry = engine_registry.read()
                            .map_err(|_| PackagesEngineError::LuaRegistryKeyLock)?;

                        let engine_table: LuaTable = lua.registry_value(engine_registry.deref())?;

                        drop(engine_registry);

                        // Load the parent resource table from the engine.
                        let resources_table = engine_table.get::<LuaTable>("resources")?;
                        let parent_resource = resources_table.get::<LuaTable>(parent_context)?;

                        // Try to parse its format and hash.
                        let parent_format = parent_resource.get::<String>("format")?;

                        let Ok(parent_format) = ResourceFormat::from_str(&parent_format) else {
                            return Err(PackagesEngineError::InvalidResourceFormat(parent_format));
                        };

                        parent_hash = Hash::from_base32(parent_resource.get::<String>("hash")?);

                        // Look into inputs of the parent resource if it's a package.
                        if parent_format == ResourceFormat::Package {
                            // Read the inputs of the parent package.
                            let parent_value = parent_resource.get::<LuaTable>("value")?;
                            let parent_inputs_table = parent_value.get::<LuaTable>("inputs")?;

                            // Iterate over input resources.
                            for resource_key in parent_inputs_table.pairs::<LuaValue, u32>() {
                                let (_, resource_key) = resource_key?;

                                // Load the requested input resource.
                                let resource = resources_table.get::<LuaTable>(resource_key)?;

                                // Try to parse its format.
                                let resource_format = resource.get::<String>("format")?;

                                let Ok(resource_format) = ResourceFormat::from_str(&resource_format) else {
                                    return Err(PackagesEngineError::InvalidResourceFormat(resource_format));
                                };

                                if matches!(resource_format, ResourceFormat::File | ResourceFormat::Archive(_)) {
                                    let path = resource.get::<String>("value")?;

                                    input_resources.push(PathBuf::from(path));
                                }
                            }
                        }
                    }

                    // Prepare special environment for the module.
                    let mut context = api::Context {
                        resource_hash: resource.lock.hash,

                        temp_folder: temp_store_path.clone(),
                        persistent_folder: persist_store_path.clone(),

                        module_folder: modules_store_path
                            .join(resource.lock.hash.to_base32()),

                        input_resources,

                        ext_process_api: false,
                        ext_allowed_paths: vec![],

                        // local_validator: local_validator.clone()
                    };

                    // Update values specified in the authority index.
                    // for hash in [Some(resource.lock.hash), parent_hash].iter().flatten() {
                    //     if let Some(ResourceStatus::Trusted { ext_process_api, allowed_paths, .. }) = validator.get_status(hash) {
                    //         if ext_process_api == Some(true) {
                    //             context.ext_process_api = true;
                    //         }

                    //         if let Some(allowed_paths) = allowed_paths {
                    //             context.ext_allowed_paths.extend(allowed_paths);
                    //         }
                    //     }
                    // }

                    #[cfg(feature = "tracing")]
                    tracing::debug!(
                        resource_url = resource.url,
                        resource_hash = ?resource.lock.hash.to_base32(),
                        parent_hash = ?parent_hash.as_ref().map(Hash::to_base32),
                        ?context,
                        "Building package module environment"
                    );

                    // Build the luau environment.
                    let env = api.create_env(&context)?;

                    // Clone the lua globals.
                    for pair in lua.globals().pairs::<LuaValue, LuaValue>() {
                        let (key, value) = pair?;

                        if !env.contains_key(&key)? {
                            env.set(key, value)?;
                        }
                    }

                    // Define standard functions depending on the standard.
                    match standard {
                        ResourceModuleFormat::Auto |
                        ResourceModuleFormat::Luau => {
                            #[cfg(feature = "tracing")]
                            tracing::trace!(
                                resource = key,
                                parent_context = ?parent_context,
                                "Indexing resource"
                            );

                            {
                                let engine_registry = engine_registry.clone();

                                env.set("load", lua.create_function(move |lua, name: String| {
                                    #[cfg(feature = "tracing")]
                                    tracing::trace!(?name, ?parent_context, "Loading package input");

                                    // Read the parent package if it exists (must be at this point).
                                    if let Some(parent_context) = parent_context {
                                        let engine_registry = engine_registry.read()
                                            .map_err(|err| LuaError::external(format!("failed to lock registry key: {err}")))?;

                                        let engine_table: LuaTable = lua.registry_value(engine_registry.deref())?;

                                        drop(engine_registry);

                                        // Load the parent resource table from the engine.
                                        let resources_table = engine_table.get::<LuaTable>("resources")?;
                                        let parent_resource = resources_table.get::<LuaTable>(parent_context)?;

                                        // Try to parse its format.
                                        let Ok(parent_format) = ResourceFormat::from_str(&parent_resource.get::<String>("format")?) else {
                                            return Err(LuaError::external("unknown parent resource format"));
                                        };

                                        // Throw an error if it's not a package type.
                                        if parent_format != ResourceFormat::Package {
                                            return Err(LuaError::external("invalid parent package format"));
                                        }

                                        // Read the inputs of the parent package.
                                        let parent_value = parent_resource.get::<LuaTable>("value")?;
                                        let parent_inputs_table = parent_value.get::<LuaTable>("inputs")?;

                                        // Try to read the requested input.
                                        if let Ok(resource_key) = parent_inputs_table.get::<u32>(name) {
                                            // Load the requested input resource.
                                            let resource = resources_table.get::<LuaTable>(resource_key)?;

                                            // Try to get its format.
                                            let Ok(format) = ResourceFormat::from_str(&resource.get::<String>("format")?) else {
                                                return Err(LuaError::external("unknown resource format"));
                                            };

                                            // If it's a package - then we have to pre-process its value.
                                            if format != ResourceFormat::Package {
                                                return Ok(resource);
                                            }

                                            // Read outputs of the package.
                                            let value = resource.get::<LuaTable>("value")?;
                                            let outputs = value.get::<LuaTable>("outputs")?;

                                            // Prepare table of filtered outputs.
                                            let filtered_resource = lua.create_table_with_capacity(0, 3)?;
                                            let filtered_outputs = lua.create_table_with_capacity(0, outputs.raw_len())?;

                                            filtered_resource.set("format", resource.get::<LuaValue>("format")?)?;
                                            filtered_resource.set("hash", resource.get::<LuaValue>("hash")?)?;
                                            filtered_resource.set("value", filtered_outputs.clone())?;

                                            // Iterate through outputs of the package.
                                            for pair in outputs.pairs::<LuaValue, u32>() {
                                                let (name, key) = pair?;

                                                filtered_outputs.set(name, resources_table.get::<LuaTable>(key)?)?;
                                            }

                                            // Return filtered package table.
                                            return Ok(filtered_resource);
                                        }
                                    }

                                    Err(LuaError::external("no resource found"))
                                })?)?;
                            }

                            {
                                let engine_registry = engine_registry.clone();

                                env.set("import", lua.create_function(move |lua, name: String| {
                                    #[cfg(feature = "tracing")]
                                    tracing::trace!(?name, ?parent_context, "Importing package input");

                                    // Read the parent package if it exists (must be at this point).
                                    if let Some(parent_context) = parent_context {
                                        let engine_registry = engine_registry.read()
                                            .map_err(|err| LuaError::external(format!("failed to lock registry key: {err}")))?;

                                        let engine_table: LuaTable = lua.registry_value(engine_registry.deref())?;

                                        drop(engine_registry);

                                        // Load the parent resource table from the engine.
                                        let resources_table = engine_table.get::<LuaTable>("resources")?;
                                        let parent_resource = resources_table.get::<LuaTable>(parent_context)?;

                                        // Try to parse its format.
                                        let Ok(parent_format) = ResourceFormat::from_str(&parent_resource.get::<String>("format")?) else {
                                            return Err(LuaError::external("unknown parent resource format"));
                                        };

                                        // Throw an error if it's not a package type.
                                        if parent_format != ResourceFormat::Package {
                                            return Err(LuaError::external("invalid parent package format"));
                                        }

                                        // Read the inputs of the parent package.
                                        let parent_value = parent_resource.get::<LuaTable>("value")?;
                                        let parent_inputs_table = parent_value.get::<LuaTable>("inputs")?;

                                        // Try to read the requested input.
                                        if let Ok(resource_key) = parent_inputs_table.get::<u32>(name) {
                                            // Load the requested input resource.
                                            let resource = resources_table.get::<LuaTable>(resource_key)?;

                                            // Try to get its format.
                                            let Ok(format) = ResourceFormat::from_str(&resource.get::<String>("format")?) else {
                                                return Err(LuaError::external("unknown resource format"));
                                            };

                                            // Read value of the resource.
                                            let value = resource.get::<LuaValue>("value")?;

                                            // If it's a package - then we have to pre-process its value.
                                            if format != ResourceFormat::Package {
                                                return Ok(value);
                                            }

                                            // Read outputs of the package.
                                            let value = resource.get::<LuaTable>("value")?;
                                            let outputs = value.get::<LuaTable>("outputs")?;

                                            // Prepare table of filtered outputs.
                                            let filtered_outputs = lua.create_table_with_capacity(0, outputs.raw_len())?;

                                            // Iterate through outputs of the package.
                                            for pair in outputs.pairs::<LuaValue, u32>() {
                                                let (name, key) = pair?;

                                                // Read the output resource.
                                                let resource = resources_table.get::<LuaTable>(key)?;

                                                // Read value of the resource.
                                                let value = resource.get::<LuaValue>("value")?;

                                                // Insert raw value of the output resource.
                                                filtered_outputs.set(name, value)?;
                                            }

                                            // Return filtered package table.
                                            return Ok(LuaValue::Table(filtered_outputs));
                                        }
                                    }

                                    Err(LuaError::external("no resource found"))
                                })?)?;
                            }
                        }
                    };

                    // Push module to the evaluation queue
                    // to execute dependencies first.
                    evaluation_queue.push((resource_table, module, env));
                }

                ResourceFormat::File |
                ResourceFormat::Archive(_) => {
                    resource_table.set("value", path.to_string_lossy())?;
                }
            }
        }

        // Enable sandbox for modules execution.
        lua.sandbox(true)?;

        // Evaluate all the modules in dependency growth order.
        for (resource_table, module, env) in evaluation_queue {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                resource_table = format!("{resource_table:#?}"),
                "Evaluating lua module"
            );

            let value = module.set_environment(env)
                .call::<LuaValue>(())?;

            resource_table.set("value", value)?;
        }

        Ok(Self {
            lua,
            engine_registry,
            lock_file,
            _api: api
        })
    }

    /// Try to load root resources from the engine.
    ///
    /// Resource keys are taken from the lock file.
    pub fn load_root_resources(&self) -> Result<Vec<LuaTable>, PackagesEngineError> {
        let engine_registry = self.engine_registry.read()
            .map_err(|_| PackagesEngineError::LuaRegistryKeyLock)?;

        let engine_table: LuaTable = self.lua.registry_value(engine_registry.deref())?;

        drop(engine_registry);

        let resources = engine_table.get::<LuaTable>("resources")?;

        let mut root = Vec::with_capacity(self.lock_file.lock.root.len());

        for key in &self.lock_file.lock.root {
            root.push(resources.get::<LuaTable>(*key)?);
        }

        Ok(root)
    }

    /// Try to load modules of the root packages from the engine.
    ///
    /// Resource keys are taken from the lock file.
    pub fn load_root_modules(&self) -> Result<Vec<LuaTable>, PackagesEngineError> {
        let engine_registry = self.engine_registry.read()
            .map_err(|_| PackagesEngineError::LuaRegistryKeyLock)?;

        let engine_table: LuaTable = self.lua.registry_value(engine_registry.deref())?;

        drop(engine_registry);

        let resources = engine_table.get::<LuaTable>("resources")?;

        let mut modules = Vec::with_capacity(self.lock_file.lock.root.len());

        // Iterate through the root resources.
        for key in &self.lock_file.lock.root {
            let resource = resources.get::<LuaTable>(*key)?;

            // If the resource is a package.
            if resource.get::<LuaString>("format")?.as_bytes() == b"package" {
                let package = resource.get::<LuaTable>("value")?;
                let outputs = package.get::<LuaTable>("outputs")?;

                // Iterate through the outputs of this package.
                for pair in outputs.pairs::<LuaValue, u32>() {
                    let (_, key) = pair?;

                    // Load the output of this package.
                    let resource = resources.get::<LuaTable>(key)?;

                    // If this output is a module - store it.
                    if resource.get::<LuaString>("format")?.as_bytes().starts_with(b"module") {
                        modules.push(resource);
                    }
                }
            }
        }

        Ok(modules)
    }

    /// Try to load resource from the engine.
    ///
    /// This function will try to find the resource
    /// by given identifier. It can be a direct index
    /// to the resource, or a hash (or a part of hash).
    pub fn load_resource(&self, identrifier: impl std::fmt::Display) -> Result<Option<LuaTable>, PackagesEngineError> {
        let engine_registry = self.engine_registry.read()
            .map_err(|_| PackagesEngineError::LuaRegistryKeyLock)?;

        let engine_table: LuaTable = self.lua.registry_value(engine_registry.deref())?;

        drop(engine_registry);

        let resources = engine_table.get::<LuaTable>("resources")?;

        let identifier = identrifier.to_string();
        let numeric_identifier = identifier.parse::<u64>().ok();

        // Try to directly load the resource.
        if let Some(index) = numeric_identifier {
            if resources.contains_key(index as u32)? {
                return Ok(Some(resources.get(index)?));
            }
        }

        // Otherwise search it through the whole list of resources.
        for (id, resource) in resources.sequence_values::<LuaTable>().enumerate() {
            let resource = resource?;

            // Check the base32 encoded hash.
            let hash = resource.get::<String>("hash")?;

            if hash.contains(&identifier) {
                return Ok(Some(resource));
            }

            // Or if can - check integer representation of the hash.
            if let Some(numeric_identifier) = numeric_identifier {
                if id as u64 == numeric_identifier {
                    return Ok(Some(resource));
                }

                if let Some(numeric_hash) = Hash::from_base32(hash) {
                    if numeric_hash.0 == numeric_identifier {
                        return Ok(Some(resource));
                    }
                }
            }
        }

        Ok(None)
    }
}

impl Drop for PackagesEngine {
    fn drop(&mut self) {
        if let Ok(mut engine_registry) = self.engine_registry.write() {
            let _ = self.lua.replace_registry_value(engine_registry.deref_mut(), LuaValue::Nil);
        }

        self.lua.expire_registry_values();

        let _ = self.lua.gc_collect();
        let _ = self.lua.gc_collect();
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn v1_standard() -> anyhow::Result<()> {
//         let path = std::env::temp_dir().join(".agl-packages-engine-test");

//         if !path.exists() {
//             std::fs::create_dir_all(&path)?;
//         }

//         let store = PackagesStore::new(&path);

//         let lock_file = LockFile::with_packages([
//             "https://raw.githubusercontent.com/an-anime-team/anime-games-launcher/next/tests/packages/1"
//         ]);

//         let lock_file = lock_file.build(&store).await
//             .map_err(|err| anyhow::anyhow!(err.to_string()))?;

//         let valid = store.validate(&lock_file)
//             .map_err(|err| anyhow::anyhow!(err.to_string()))?;

//         assert!(valid);

//         let validator = AuthorityValidator::new([]);
//         let local_validator = LocalValidator::open(std::env::temp_dir().join("local_validator.json"))?;

//         let engine = PackagesEngine::create(
//             Lua::new(),
//             &store,
//             lock_file,
//             validator,
//             local_validator,
//             PackagesEngineOptions::default()
//         )?;

//         let resource = engine.load_resource("0peottaa6s1co")?
//             .ok_or_else(|| anyhow::anyhow!("Module expected, got none"))?;

//         let value = resource.get::<LuaTable>("value")?;

//         assert!(value.get::<bool>("test_load_valid_input")?);
//         assert!(!value.get::<bool>("test_load_valid_output")?);
//         assert!(!value.get::<bool>("test_load_invalid_input")?);
//         assert!(!value.get::<bool>("test_load_invalid_output")?);
//         assert!(!value.get::<bool>("test_load_unexisting_input")?);

//         assert_eq!(value.call_function::<String>("greeting", ())?, "Hello, World!");

//         Ok(())
//     }
// }
