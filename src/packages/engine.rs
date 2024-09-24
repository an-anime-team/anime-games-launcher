use std::collections::{HashSet, VecDeque};
use std::str::FromStr;

use mlua::prelude::*;

use crate::packages::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Lua engine error: {0}")]
    Lua(#[from] LuaError)
}

#[derive(Debug)]
pub struct Engine<'lua> {
    lua: &'lua Lua,
    store: PackagesStore,
    lock_file: LockFileManifest
}

impl<'lua> Engine<'lua> {
    /// Create new packages engine and load all the resources
    /// from the provided lock file.
    pub fn create(lua: &'lua Lua, store: PackagesStore, lock_file: LockFileManifest) -> Result<Self, EngineError> {
        let engine_table = lua.create_table()?;
        let resources_table = lua.create_table()?;

        // Lua tables are shared (like in JS) so I can store them right there.
        engine_table.set("resources", resources_table.clone())?;

        lua.globals().set("#!ENGINE", engine_table.clone())?;

        let mut resources = VecDeque::with_capacity(lock_file.resources.len());
        let mut visited_resources = HashSet::new();
        let mut evaluation_queue = VecDeque::with_capacity(lock_file.resources.len());

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

                    package.set("inputs", inputs_table)?;
                    package.set("outputs", outputs_table)?;

                    resource_table.set("value", package)?;
                }

                PackageResourceFormat::Module(standard) => {
                    let module = std::fs::read(&path)?;
                    let module = lua.load(module);

                    // Prepare special environment for the module.
                    let env = lua.create_table()?;

                    // Clone the lua globals.
                    for pair in lua.globals().pairs::<LuaValue, LuaValue>() {
                        let (key, value) = pair?;

                        if key.as_str() != Some("#!ENGINE") {
                            env.set(key, value)?;
                        }
                    }

                    env.set("#!ENGINE", engine_table.clone())?;

                    // Define standard functions depending on the standard.
                    match standard {
                        PackageResourceModuleStandard::Auto |
                        PackageResourceModuleStandard::V1 => {
                            env.set("load", lua.create_function(move |lua, name: String| {
                                // Load the engine table.
                                let engine = lua.globals().get::<_, LuaTable>("#!ENGINE")?;

                                // Read the parent package if it exists (must be at this point).
                                if let Some(parent_context) = parent_context {
                                    // Load the parent resource table from the engine.
                                    let resources_table = engine.get::<_, LuaTable>("resources")?;
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
                                    if let Ok(resource_key) = parent_inputs_table.get::<_, u64>(name) {
                                        return resources_table.get::<_, LuaTable>(resource_key);
                                    }
                                }

                                Err(LuaError::external("no resource found"))
                            })?)?;
                        }
                    };

                    // Push module to the evaluation queue
                    // to execute dependencies first.
                    evaluation_queue.push_back((resource_table, module, env));
                }

                PackageResourceFormat::File |
                PackageResourceFormat::Archive(_) => {
                    resource_table.set("value", path.to_string_lossy())?;
                }
            }
        }

        // Evaluate all the modules in dependency growth order.
        while let Some((resource_table, module, env)) = evaluation_queue.pop_front() {
            let value = module.set_environment(env)
                .call::<_, LuaTable>(())?;

            resource_table.set("value", value)?;
        }

        Ok(Self {
            lua,
            store,
            lock_file
        })
    }

    /// Try to load root resources from the engine.
    /// 
    /// Resource keys are taken from the lock file.
    pub fn load_root_resources(&'lua self) -> Result<Vec<LuaTable<'lua>>, EngineError> {
        let engine = self.lua.globals().get::<_, LuaTable>("#!ENGINE")?;
        let resources = engine.get::<_, LuaTable>("resources")?;

        let mut root = Vec::with_capacity(self.lock_file.root.len());

        for key in &self.lock_file.root {
            root.push(resources.get::<_, LuaTable>(*key)?);
        }

        Ok(root)
    }

    /// Try to load resource from the engine.
    ///
    /// This function will try to find the resource
    /// by given identifier. It can be a direct index
    /// to the resource, or a hash (or a part of hash).
    pub fn load_resource(&'lua self, identrifier: impl std::fmt::Display) -> Result<Option<LuaTable<'lua>>, EngineError> {
        let engine = self.lua.globals().get::<_, LuaTable>("#!ENGINE")?;
        let resources = engine.get::<_, LuaTable>("resources")?;

        let identifier = identrifier.to_string();
        let numeric_identifier = identifier.parse::<u64>().ok();
        
        // Try to directly load the resource.
        if let Some(index) = numeric_identifier {
            if resources.contains_key(index)? {
                return Ok(Some(resources.get(index)?));
            }
        }

        // Otherwise search it through the whole list of resources.
        for resource in resources.sequence_values::<LuaTable>() {
            let resource = resource?;

            // Check the base32 encoded hash.
            let hash = resource.get::<_, String>("hash")?;

            if hash.contains(&identifier) {
                return Ok(Some(resource));
            }

            // Or if can - check integer representation of the hash.
            if let Some(numeric_identifier) = numeric_identifier {
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

        let engine = Engine::create(&lua, store, lock_file)?;

        let resource = engine.load_resource("gqj0qechfgcge")?
            .ok_or_else(|| anyhow::anyhow!("Module expected, got none"))?;

        let value = resource.get::<_, LuaTable>("value")?;

        assert!(value.get::<_, bool>("test_load_valid_input")?);
        assert!(!value.get::<_, bool>("test_load_valid_output")?);
        assert!(!value.get::<_, bool>("test_load_invalid_input")?);
        assert!(!value.get::<_, bool>("test_load_invalid_output")?);
        assert!(!value.get::<_, bool>("test_load_unexisting_input")?);

        assert_eq!(value.get::<_, String>("greeting")?, "Hello, World!");

        Ok(())
    }
}
