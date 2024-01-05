use std::collections::HashMap;

use mlua::prelude::*;

use super::IntegrationStandard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edition {
    pub name: String,
    pub title: String
}

impl Edition {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    name: table.get::<_, String>("name")?,
                    title: table.get::<_, String>("title")?
                })
            }
        }
    }

    pub fn to_table<'a>(&self, lua: &'a Lua, standard: IntegrationStandard) -> anyhow::Result<LuaTable<'a>> {
        match standard {
            IntegrationStandard::V1 => {
                let table = lua.create_table()?;

                table.set("name", self.name.as_str())?;
                table.set("title", self.title.as_str())?;

                Ok(table)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchOptions {
    pub executable: String,
    pub environment: HashMap<String, String>
}

impl LaunchOptions {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    executable: table.get::<_, String>("executable")?,
                    environment: table.get::<_, LuaTable>("environment")?
                        .pairs::<String, String>()
                        .flatten()
                        .collect()
                })
            }
        }
    }

    pub fn to_table<'a>(&self, lua: &'a Lua, standard: IntegrationStandard) -> anyhow::Result<LuaTable<'a>> {
        match standard {
            IntegrationStandard::V1 => {
                let table = lua.create_table()?;
                let environment = lua.create_table()?;

                for (key, value) in self.environment.clone() {
                    environment.set(key, value)?;
                }

                table.set("executable", self.executable.as_str())?;
                table.set("environment", environment)?;

                Ok(table)
            }
        }
    }
}
