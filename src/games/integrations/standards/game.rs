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
pub struct Status {
    pub allow_launch: bool,
    pub severity: StatusSeverity,
    pub reason: Option<String>
}

impl Status {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    allow_launch: table.get::<_, bool>("allow_launch")?,

                    severity: StatusSeverity::from_str(table.get::<_, String>("severity")?, standard)?,

                    reason: table.contains_key("reason")?
                        .then(|| table.get::<_, Option<String>>("reason"))
                        .unwrap_or(Ok(None))?
                })
            }
        }
    }

    pub fn to_table<'a>(&self, lua: &'a Lua, standard: IntegrationStandard) -> anyhow::Result<LuaTable<'a>> {
        match standard {
            IntegrationStandard::V1 => {
                let table = lua.create_table()?;

                table.set("allow_launch", self.allow_launch)?;
                table.set("severity", self.severity.to_str(standard))?;

                if let Some(reason) = &self.reason {
                    table.set("reason", reason.as_str())?;
                }

                Ok(table)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusSeverity {
    Critical,
    Warning,
    None
}

impl StatusSeverity {
    pub fn from_str(value: impl AsRef<str>, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                match value.as_ref() {
                    "critical" => Ok(Self::Critical),
                    "warning"  => Ok(Self::Warning),
                    "none"     => Ok(Self::None),

                    _ => anyhow::bail!("Wrong v1 status severity: '{}'", value.as_ref())
                }
            }
        }
    }

    pub fn to_str(&self, standard: IntegrationStandard) -> &str {
        match standard {
            IntegrationStandard::V1 => {
                match self {
                    Self::Critical => "critical",
                    Self::Warning  => "warning",
                    Self::None     => "none"
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchOptions {
    pub executable: String,
    pub options: Vec<String>,
    pub environment: HashMap<String, String>
}

impl LaunchOptions {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    executable: table.get::<_, String>("executable")?,

                    options: table.get::<_, LuaTable>("environment")?
                        .sequence_values::<String>()
                        .flatten()
                        .collect(),

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

                let options = lua.create_table()?;
                let environment = lua.create_table()?;

                for option in &self.options {
                    environment.push(option.as_str())?;
                }

                for (key, value) in &self.environment {
                    environment.set(key.as_str(), value.as_str())?;
                }

                table.set("executable", self.executable.as_str())?;
                table.set("options", options)?;
                table.set("environment", environment)?;

                Ok(table)
            }
        }
    }
}
