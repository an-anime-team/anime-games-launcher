use mlua::prelude::*;

use super::IntegrationStandard;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AddonsGroup {
    pub name: String,
    pub title: String,
    pub addons: Vec<Addon>
}

impl AddonsGroup {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    name: table.get::<_, String>("name")?,
                    title: table.get::<_, String>("title")?,

                    addons: table.get::<_, LuaTable>("addons")?
                        .sequence_values()
                        .flatten()
                        .flat_map(|addon| Addon::from_table(addon, standard))
                        .collect()
                })
            }
        }
    }

    pub fn to_table<'a>(&self, lua: &'a Lua, standard: IntegrationStandard) -> anyhow::Result<LuaTable<'a>> {
        match standard {
            IntegrationStandard::V1 => {
                let table = lua.create_table()?;
                let addons = lua.create_table()?;

                for addon in &self.addons {
                    addons.push(addon.to_table(lua, standard)?)?;
                }

                table.set("name", self.name.as_str())?;
                table.set("title", self.title.as_str())?;
                table.set("addons", addons)?;

                Ok(table)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Addon {
    pub r#type: AddonType,
    pub name: String,
    pub title: String,
    pub version: String,
    pub required: bool
}

impl Addon {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    r#type: AddonType::from_str(table.get::<_, String>("type")?, standard)?,
                    name: table.get::<_, String>("name")?,
                    title: table.get::<_, String>("title")?,
                    version: table.get::<_, String>("version")?,
                    required: table.get::<_, bool>("required")?
                })
            }
        }
    }

    pub fn to_table<'a>(&self, lua: &'a Lua, standard: IntegrationStandard) -> anyhow::Result<LuaTable<'a>> {
        match standard {
            IntegrationStandard::V1 => {
                let table = lua.create_table()?;

                table.set("type", self.r#type.to_str(standard))?;
                table.set("name", self.name.as_str())?;
                table.set("title", self.title.as_str())?;
                table.set("version", self.version.as_str())?;
                table.set("required", self.required)?;

                Ok(table)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddonType {
    Module,
    Component
}

impl AddonType {
    pub fn from_str(value: impl AsRef<str>, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                match value.as_ref() {
                    "module"    => Ok(Self::Module),
                    "component" => Ok(Self::Component),

                    _ => anyhow::bail!("Wrong v1 addon type: '{}'", value.as_ref())
                }
            }
        }
    }

    pub fn to_str(&self, standard: IntegrationStandard) -> &str {
        match standard {
            IntegrationStandard::V1 => {
                match self {
                    Self::Module    => "module",
                    Self::Component => "component"
                }
            }
        }
    }
}
