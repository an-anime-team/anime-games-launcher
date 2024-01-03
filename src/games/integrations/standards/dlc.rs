use mlua::prelude::*;

use super::IntegrationStandard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub name: String,
    pub title: String,
    pub dlcs: Vec<Dlc>
}

impl Group {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    name: table.get::<_, String>("name")?,
                    title: table.get::<_, String>("title")?,
                    dlcs: table.get::<_, LuaTable>("dlcs")?
                        .sequence_values()
                        .flatten()
                        .flat_map(|dlc| Dlc::from_table(dlc, standard))
                        .collect()
                })
            }
        }
    }

    pub fn to_table<'a>(&self, lua: &'a Lua, standard: IntegrationStandard) -> anyhow::Result<LuaTable<'a>> {
        match standard {
            IntegrationStandard::V1 => {
                let table = lua.create_table()?;
                let dlcs = lua.create_table()?;

                for dlc in &self.dlcs {
                    dlcs.push(dlc.to_table(lua, standard)?)?;
                }

                table.set("name", self.name.as_str())?;
                table.set("title", self.title.as_str())?;
                table.set("dlcs", dlcs)?;

                Ok(table)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dlc {
    pub name: String,
    pub title: String,
    pub required: bool
}

impl Dlc {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    name: table.get::<_, String>("name")?,
                    title: table.get::<_, String>("title")?,
                    required: table.get::<_, bool>("required")?
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
                table.set("required", self.required)?;

                Ok(table)
            }
        }
    }
}
