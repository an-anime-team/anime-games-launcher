use mlua::prelude::*;

use super::IntegrationStandard;
use super::diff::DiffFileDownload;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntegrityInfo {
    pub hash: HashType,
    pub value: String,
    pub file: DiffFileDownload
}

impl IntegrityInfo {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    hash: HashType::from_str(table.get::<_, String>("hash")?, standard)?,
                    value: table.get::<_, String>("value")?,
                    file: DiffFileDownload::from_table(table.get::<_, LuaTable>("file")?, standard)?
                })
            }
        }
    }

    pub fn to_table<'a>(&self, lua: &'a Lua, standard: IntegrationStandard) -> anyhow::Result<LuaTable<'a>> {
        match standard {
            IntegrationStandard::V1 => {
                let table = lua.create_table()?;

                table.set("hash", self.hash.to_str(standard))?;
                table.set("value", self.value.as_str())?;
                table.set("file", self.file.to_table(lua, standard)?)?;

                Ok(table)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HashType {
    Md5,
    Sha1,
    Crc32,
    Xxhash32,
    Xxhash64,
    Custom(String)
}

impl HashType {
    pub fn from_str(value: impl AsRef<str>, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                match value.as_ref() {
                    "md5"      => Ok(Self::Md5),
                    "sha1"     => Ok(Self::Sha1),
                    "crc32"    => Ok(Self::Crc32),
                    "xxhash32" => Ok(Self::Xxhash32),
                    "xxhash64" => Ok(Self::Xxhash64),

                    name => Ok(Self::Custom(name.to_string()))
                }
            }
        }
    }

    pub fn to_str(&self, standard: IntegrationStandard) -> &str {
        match standard {
            IntegrationStandard::V1 => {
                match self {
                    Self::Md5      => "md5",
                    Self::Sha1     => "sha1",
                    Self::Crc32    => "crc32",
                    Self::Xxhash32 => "xxhash32",
                    Self::Xxhash64 => "xxhash64",

                    Self::Custom(name) => name
                }
            }
        }
    }
}
