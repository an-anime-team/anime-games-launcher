use mlua::prelude::*;

use super::IntegrationStandard;
use super::diff::DiffInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Download {
    pub version: String,
    pub edition: String,
    pub download: DiffInfo
}

impl Download {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    version: table.get::<_, String>("version")?,
                    edition: table.get::<_, String>("edition")?,
                    download: DiffInfo::from_table(table.get::<_, LuaTable>("download")?, standard)?
                })
            }
        }
    }

    pub fn to_table<'a>(&self, lua: &'a Lua, standard: IntegrationStandard) -> anyhow::Result<LuaTable<'a>> {
        match standard {
            IntegrationStandard::V1 => {
                let table = lua.create_table()?;

                table.set("version", self.version.clone())?;
                table.set("edition", self.edition.clone())?;
                table.set("download", self.download.to_table(lua, standard)?)?;

                Ok(table)
            }
        }
    }
}
