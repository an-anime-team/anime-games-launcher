use mlua::prelude::*;

use super::IntegrationStandard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    pub status: Status,
    pub progress: Progress
}

impl Info {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    status: Status::from_str(table.get::<_, String>("status")?, standard)?,
                    progress: Progress::from_table(table.get::<_, LuaTable>("progress")?, standard)?
                })
            }
        }
    }

    pub fn to_table<'a>(&self, lua: &'a Lua, standard: IntegrationStandard) -> anyhow::Result<LuaTable<'a>> {
        match standard {
            IntegrationStandard::V1 => {
                let table = lua.create_table()?;

                table.set("status", self.status.to_str(standard))?;
                table.set("progress", self.progress.to_table(lua, standard)?)?;

                Ok(table)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Suspended,
    Running,
    Dead
}

impl Status {
    pub fn from_str(value: impl AsRef<str>, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                match value.as_ref() {
                    "suspended" => Ok(Self::Suspended),
                    "running"   => Ok(Self::Running),
                    "dead"      => Ok(Self::Dead),

                    _ => anyhow::bail!("Wrong v1 task status: {}", value.as_ref())
                }
            }
        }
    }

    pub fn to_str(&self, standard: IntegrationStandard) -> &str {
        match standard {
            IntegrationStandard::V1 => {
                match self {
                    Self::Suspended => "suspended",
                    Self::Running   => "running",
                    Self::Dead      => "dead"
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    pub current: u64,
    pub total: u64,
    pub label: String
}

impl Progress {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    current: table.get::<_, u64>("current")?,
                    total: table.get::<_, u64>("total")?,
                    label: table.get::<_, String>("label")?
                })
            }
        }
    }

    pub fn to_table<'a>(&self, lua: &'a Lua, standard: IntegrationStandard) -> anyhow::Result<LuaTable<'a>> {
        match standard {
            IntegrationStandard::V1 => {
                let table = lua.create_table()?;

                table.set("current", self.current)?;
                table.set("total", self.total)?;
                table.set("label", self.label.as_str())?;

                Ok(table)
            }
        }
    }
}
