use std::collections::{VecDeque, HashSet};

use mlua::prelude::*;

use crate::packages::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Lua engine error: {0}")]
    Lua(#[from] LuaError),

    #[error("Failed to send data to the packages engine channel: {0}")]
    Send(String),

    #[error("Failed to receive data from packages engine channel: {0}")]
    Receive(String)
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

        let resources_table = lua.create_table()?; // [hash][format] => resource
        let context_map_table = lua.create_table()?; // [hash] => <inputs, outputs>

        let mut resources = VecDeque::with_capacity(lock_file.resources.len());
        let mut visited_resources = HashSet::new();

        for root in &lock_file.root {
            resources.push_back((*root, None));
            visited_resources.insert(*root);
        }

        while let Some((key, parent_context)) = resources.pop_front() {
            let resource = &lock_file.resources[key as usize];

            let path = store.get_path(&resource.lock.hash, &resource.format);

            let resource_table = lua.create_table()?;

            resource_table.set("format", resource.format.to_string())?;
            resource_table.set("hash", resource.lock.hash.to_base32())?;

            match resource.format {
                PackageResourceFormat::Package => {
                    let package = lua.create_table()?;
                    let inputs_table = lua.create_table()?;
                    let outputs_table = lua.create_table()?;

                    if let Some(inputs) = &resource.inputs {
                        for (name, input_key) in inputs.clone() {
                            inputs_table.set(name, input_key)?;

                            if visited_resources.insert(input_key) {
                                resources.push_back((key, None));
                            }
                        }
                    }

                    if let Some(outputs) = &resource.outputs {
                        for (name, output_key) in outputs.clone() {
                            outputs_table.set(name, output_key)?;

                            if visited_resources.insert(output_key) {
                                resources.push_back((key, Some(key)));
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

                    // Clone _ENV ?
                    let env = lua.create_table()?;

                    env.set("__engine", engine_table.clone())?;

                    match standard {
                        PackageResourceModuleStandard::Auto |
                        PackageResourceModuleStandard::V1 => {
                            env.set("load", {
                                lua.create_function(move |lua, name: String| {
                                    let engine = lua.globals().get::<_, LuaTable>("__engine")?;

                                    if let Some(parent_context) = parent_context {
                                        let resources_table = engine.get::<_, LuaTable>("resources")?;
                                        let parent_resource = resources_table.get::<_, LuaTable>(parent_context)?;

                                        let parent_inputs_table = parent_resource.get::<_, LuaTable>("inputs")?;

                                        if let Ok(resource_key) = parent_inputs_table.get::<_, u64>(name) {
                                            return resources_table.get::<_, LuaTable>(resource_key);
                                        }
                                    }

                                    Err(LuaError::external("no resource found"))
                                })?
                            })?;
                        }
                    };

                    let module = module.set_environment(env)
                        .eval::<LuaTable>()?;

                    resource_table.set("value", module)?;
                }

                PackageResourceFormat::File |
                PackageResourceFormat::Archive(_) => {
                    resource_table.set("value", path.to_string_lossy())?;
                }
            }

            resources_table.set(key, resource_table)?;
        }

        engine_table.set("resources", resources_table)?;
        engine_table.set("context_map", context_map_table)?;

        lua.globals().set("__engine", engine_table)?;

        Ok(Self {
            lua,
            store,
            lock_file
        })
    }

    /// Try to load resource from the engine.
    ///
    /// Resource key is taken from the lock file.
    pub fn load_resource(&'lua self, index: u64) -> Result<Option<LuaTable<'lua>>, EngineError> {
        let engine = self.lua.globals().get::<_, LuaTable>("__engine")?;
        let resources = engine.get::<_, LuaTable>("resources")?;

        if !resources.contains_key(index)? {
            return Ok(None);
        }

        Ok(Some(resources.get::<_, LuaTable>(index)?))
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

        let engine = Engine::create(&lua, store, lock_file)
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        let resource = engine.load_resource(0)
            .map_err(|err| anyhow::anyhow!(err.to_string()))?
            .ok_or_else(|| anyhow::anyhow!("Module expected, got none"))?;

        let greeting = resource.get::<_, String>("greeting")
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        assert_eq!(greeting, "Hello, World!");

        Ok(())
    }
}
