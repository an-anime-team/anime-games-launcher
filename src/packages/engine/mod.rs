use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::str::FromStr;

use mlua::prelude::*;

use crate::packages::prelude::*;
use crate::config;

pub mod v1_standard;

#[derive(Debug, thiserror::Error)]
pub enum PackagesEngineError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Lua engine error: {0}")]
    Lua(#[from] LuaError),

    #[error("Network error: {0}")]
    Reqwest(#[from] reqwest::Error)
}

pub struct PackagesEngine<'lua> {
    lua: &'lua Lua,
    engine_registry: Arc<LuaRegistryKey>,
    lock_file: LockFileManifest,

    _v1_standard: v1_standard::Standard<'lua>
}

impl<'lua> PackagesEngine<'lua> {
    /// Create new packages engine and load all the resources
    /// from the provided lock file.
    pub fn create(lua: &'lua Lua, store: &PackagesStore, lock_file: LockFileManifest) -> Result<Self, PackagesEngineError> {
        let engine_table = lua.create_table()?;
        let resources_table = lua.create_table()?;

        // Lua tables are shared (like in JS) so I can store them right there.
        engine_table.set("resources", resources_table.clone())?;

        let engine_registry = Arc::new(lua.create_registry_value(engine_table)?);

        let mut resources = VecDeque::with_capacity(lock_file.resources.len());
        let mut visited_resources = HashSet::new();
        let mut evaluation_queue = Vec::with_capacity(lock_file.resources.len());

        // Prepare standard folders.
        let config = config::get();

        let temp_folder = std::env::temp_dir()
            .join(".agl-modules-temp"); // TODO: make a separate launcher-global temp folder

        if !config.packages.persist_store.path.exists() {
            std::fs::create_dir_all(&config.packages.persist_store.path)?;
        }

        if !temp_folder.exists() {
            std::fs::create_dir_all(&temp_folder)?;
        }

        // Prepare modules standard implementations.
        let v1_standard = v1_standard::Standard::new(lua)?;

        // Push root resources to the processing queue.
        for root in &lock_file.root {
            resources.push_back((*root, None));
            visited_resources.insert(*root);
        }

        // Iterate over all the locked resources.
        while let Some((key, parent_context)) = resources.pop_front() {
            // Resolve base resource info.
            let resource = &lock_file.resources[key as usize];

            let path = store.get_path(&resource.lock.hash, &resource.format);

            // Store basic info to the lua representation.
            let resource_table = lua.create_table()?;

            resource_table.set("format", resource.format.to_string())?;
            resource_table.set("hash", resource.lock.hash.to_base32())?;

            resources_table.set(key, resource_table.clone())?;

            // Prepare resource value depending on its format.
            match resource.format {
                PackageResourceFormat::Package => {
                    let package = lua.create_table()?;

                    let inputs_table = lua.create_table()?;
                    let outputs_table = lua.create_table()?;

                    package.set("inputs", inputs_table.clone())?;
                    package.set("outputs", outputs_table.clone())?;

                    resource_table.set("value", package.clone())?;

                    // Load inputs and outputs of the package,
                    // push to the queue which weren't processed yet.
                    if let Some(inputs) = &resource.inputs {
                        for (name, input_key) in inputs.clone() {
                            inputs_table.set(name, input_key)?;

                            if visited_resources.insert(input_key) {
                                // Do not reference this package for inputs
                                // because inputs can't load other inputs.
                                resources.push_back((input_key, None));
                            }
                        }
                    }

                    if let Some(outputs) = &resource.outputs {
                        for (name, output_key) in outputs.clone() {
                            outputs_table.set(name, output_key)?;

                            if visited_resources.insert(output_key) {
                                // Reference this package so the output module
                                // can load inputs of this package.
                                resources.push_back((output_key, Some(key)));
                            }
                        }
                    }
                }

                PackageResourceFormat::Module(standard) => {
                    let module = std::fs::read(&path)?;
                    let module = lua.load(module);

                    // Prepare the module's context.
                    let module_folder = config.packages.modules_store.path
                        .join(resource.lock.hash.to_base32());

                    if !module_folder.exists() {
                        std::fs::create_dir_all(&module_folder)?;
                    }

                    // Prepare special environment for the module.
                    let env = v1_standard.create_env(&v1_standard::Context {
                        temp_folder: temp_folder.clone(),
                        module_folder,
                        persistent_folder: config.packages.persist_store.path.clone(),

                        // TODO: implement packages authorities system
                        ext_process_api: false
                    })?;

                    // Clone the lua globals.
                    for pair in lua.globals().pairs::<LuaValue, LuaValue>() {
                        let (key, value) = pair?;

                        if !env.contains_key(&key)? {
                            env.set(key, value)?;
                        }
                    }

                    // Define standard functions depending on the standard.
                    match standard {
                        PackageResourceModuleStandard::Auto |
                        PackageResourceModuleStandard::V1 => {
                            tracing::trace!("Indexing resource {key} with parent context {parent_context:?}");

                            {
                                let engine_registry = engine_registry.clone();

                                env.set("load", lua.create_function(move |lua, name: String| {
                                    tracing::trace!(?name, ?parent_context, "Loading package input");

                                    // Read the parent package if it exists (must be at this point).
                                    if let Some(parent_context) = parent_context {
                                        let engine_table: LuaTable = lua.registry_value(&engine_registry)?;

                                        // Load the parent resource table from the engine.
                                        let resources_table = engine_table.get::<_, LuaTable>("resources")?;
                                        let parent_resource = resources_table.get::<_, LuaTable>(parent_context)?;

                                        // Try to parse its format.
                                        let Ok(parent_format) = PackageResourceFormat::from_str(&parent_resource.get::<_, String>("format")?) else {
                                            return Err(LuaError::external("unknown parent resource format"));
                                        };

                                        // Throw an error if it's not a package type.
                                        if parent_format != PackageResourceFormat::Package {
                                            return Err(LuaError::external("invalid parent package format"));
                                        }

                                        // Read the inputs of the parent package.
                                        let parent_value = parent_resource.get::<_, LuaTable>("value")?;
                                        let parent_inputs_table = parent_value.get::<_, LuaTable>("inputs")?;

                                        // Try to read the requested input.
                                        if let Ok(resource_key) = parent_inputs_table.get::<_, u32>(name) {
                                            // Load the requested input resource.
                                            let resource = resources_table.get::<_, LuaTable>(resource_key)?;

                                            // Try to get its format.
                                            let Ok(format) = PackageResourceFormat::from_str(&resource.get::<_, String>("format")?) else {
                                                return Err(LuaError::external("unknown resource format"));
                                            };

                                            // If it's a package - then we have to pre-process its value.
                                            if format != PackageResourceFormat::Package {
                                                return Ok(resource);
                                            }

                                            // Read outputs of the package.
                                            let value = resource.get::<_, LuaTable>("value")?;
                                            let outputs = value.get::<_, LuaTable>("outputs")?;

                                            // Prepare table of filtered outputs.
                                            let filtered_resource = lua.create_table_with_capacity(0, 3)?;
                                            let filtered_outputs = lua.create_table_with_capacity(0, outputs.raw_len())?;

                                            filtered_resource.set("format", resource.get::<_, LuaValue>("format")?)?;
                                            filtered_resource.set("hash", resource.get::<_, LuaValue>("hash")?)?;
                                            filtered_resource.set("value", filtered_outputs.clone())?;

                                            // Iterate through outputs of the package.
                                            for pair in outputs.pairs::<LuaValue, u32>() {
                                                let (name, key) = pair?;

                                                filtered_outputs.set(name, resources_table.get::<_, LuaTable>(key)?)?;
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
                                    tracing::trace!(?name, ?parent_context, "Importing package input");

                                    // Read the parent package if it exists (must be at this point).
                                    if let Some(parent_context) = parent_context {
                                        let engine_table: LuaTable = lua.registry_value(&engine_registry)?;

                                        // Load the parent resource table from the engine.
                                        let resources_table = engine_table.get::<_, LuaTable>("resources")?;
                                        let parent_resource = resources_table.get::<_, LuaTable>(parent_context)?;

                                        // Try to parse its format.
                                        let Ok(parent_format) = PackageResourceFormat::from_str(&parent_resource.get::<_, String>("format")?) else {
                                            return Err(LuaError::external("unknown parent resource format"));
                                        };

                                        // Throw an error if it's not a package type.
                                        if parent_format != PackageResourceFormat::Package {
                                            return Err(LuaError::external("invalid parent package format"));
                                        }

                                        // Read the inputs of the parent package.
                                        let parent_value = parent_resource.get::<_, LuaTable>("value")?;
                                        let parent_inputs_table = parent_value.get::<_, LuaTable>("inputs")?;

                                        // Try to read the requested input.
                                        if let Ok(resource_key) = parent_inputs_table.get::<_, u32>(name) {
                                            // Load the requested input resource.
                                            let resource = resources_table.get::<_, LuaTable>(resource_key)?;

                                            // Try to get its format.
                                            let Ok(format) = PackageResourceFormat::from_str(&resource.get::<_, String>("format")?) else {
                                                return Err(LuaError::external("unknown resource format"));
                                            };

                                            // Read value of the resource.
                                            let value = resource.get::<_, LuaValue>("value")?;

                                            // If it's a package - then we have to pre-process its value.
                                            if format != PackageResourceFormat::Package {
                                                return Ok(value);
                                            }

                                            // Read outputs of the package.
                                            let value = resource.get::<_, LuaTable>("value")?;
                                            let outputs = value.get::<_, LuaTable>("outputs")?;

                                            // Prepare table of filtered outputs.
                                            let filtered_outputs = lua.create_table_with_capacity(0, outputs.raw_len())?;

                                            // Iterate through outputs of the package.
                                            for pair in outputs.pairs::<LuaValue, u32>() {
                                                let (name, key) = pair?;

                                                // Read the output resource.
                                                let resource = resources_table.get::<_, LuaTable>(key)?;

                                                // Read value of the resource.
                                                let value = resource.get::<_, LuaValue>("value")?;

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

                PackageResourceFormat::File |
                PackageResourceFormat::Archive(_) => {
                    resource_table.set("value", path.to_string_lossy())?;
                }
            }
        }

        // Enable sandbox for modules execution.
        lua.sandbox(true)?;

        // Evaluate all the modules in dependency growth order.
        while let Some((resource_table, module, env)) = evaluation_queue.pop() {
            tracing::trace!(resource_table = format!("{resource_table:#?}"), "Evaluating lua module");

            let value = module.set_environment(env)
                .call::<_, LuaValue>(())?;

            resource_table.set("value", value)?;
        }

        Ok(Self {
            lua,
            engine_registry,
            lock_file,

            _v1_standard: v1_standard
        })
    }

    /// Try to load root resources from the engine.
    ///
    /// Resource keys are taken from the lock file.
    pub fn load_root_resources(&self) -> Result<Vec<LuaTable<'lua>>, PackagesEngineError> {
        let engine_table: LuaTable = self.lua.registry_value(&self.engine_registry)?;
        let resources = engine_table.get::<_, LuaTable>("resources")?;

        let mut root = Vec::with_capacity(self.lock_file.root.len());

        for key in &self.lock_file.root {
            root.push(resources.get::<_, LuaTable>(*key)?);
        }

        Ok(root)
    }

    /// Try to load modules of the root packages
    /// from the engine.
    ///
    /// Resource keys are taken from the lock file.
    pub fn load_root_modules(&self) -> Result<Vec<LuaTable<'lua>>, PackagesEngineError> {
        let engine_table: LuaTable = self.lua.registry_value(&self.engine_registry)?;
        let resources = engine_table.get::<_, LuaTable>("resources")?;

        let mut modules = Vec::with_capacity(self.lock_file.root.len());

        // Iterate through the root resources.
        for key in &self.lock_file.root {
            let resource = resources.get::<_, LuaTable>(*key)?;

            // If the resource is a package.
            if resource.get::<_, LuaString>("format")?.as_bytes() == b"package" {
                let package = resource.get::<_, LuaTable>("value")?;
                let outputs = package.get::<_, LuaTable>("outputs")?;

                // Iterate through the outputs of this package.
                for pair in outputs.pairs::<LuaValue, u32>() {
                    let (_, key) = pair?;

                    // Load the output of this package.
                    let resource = resources.get::<_, LuaTable>(key)?;

                    // If this output is a module - store it.
                    if resource.get::<_, LuaString>("format")?.as_bytes().starts_with(b"module") {
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
    pub fn load_resource(&self, identrifier: impl std::fmt::Display) -> Result<Option<LuaTable<'lua>>, PackagesEngineError> {
        let engine_table: LuaTable = self.lua.registry_value(&self.engine_registry)?;
        let resources = engine_table.get::<_, LuaTable>("resources")?;

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
            let hash = resource.get::<_, String>("hash")?;

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

impl Drop for PackagesEngine<'_> {
    fn drop(&mut self) {
        let _ = self.lua.replace_registry_value(&self.engine_registry, LuaNil);

        self.lua.expire_registry_values();

        let _ = self.lua.gc_collect();
        let _ = self.lua.gc_collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn v1_standard() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-packages-engine-test");

        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        let store = PackagesStore::new(&path);

        let lock_file = LockFile::with_packages([
            "https://raw.githubusercontent.com/an-anime-team/anime-games-launcher/next/tests/packages/1"
        ]);

        let lock_file = lock_file.build(&store).await
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        let valid = store.validate(&lock_file)
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        assert!(valid);

        let lua = Lua::new();

        let engine = PackagesEngine::create(&lua, &store, lock_file)?;

        let resource = engine.load_resource("0peottaa6s1co")?
            .ok_or_else(|| anyhow::anyhow!("Module expected, got none"))?;

        let value = resource.get::<_, LuaTable>("value")?;

        assert!(value.get::<_, bool>("test_load_valid_input")?);
        assert!(!value.get::<_, bool>("test_load_valid_output")?);
        assert!(!value.get::<_, bool>("test_load_invalid_input")?);
        assert!(!value.get::<_, bool>("test_load_invalid_output")?);
        assert!(!value.get::<_, bool>("test_load_unexisting_input")?);

        assert_eq!(value.call_function::<_, String>("greeting", ())?, "Hello, World!");

        Ok(())
    }
}
