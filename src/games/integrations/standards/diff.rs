use mlua::prelude::*;

use super::IntegrationStandard;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    pub current_version: String,
    pub latest_version: String,
    pub edition: String,
    pub status: DiffStatus,
    pub diff: Option<DiffInfo>
}

impl Diff {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                Ok(Self {
                    current_version: table.get::<_, String>("current_version")?,
                    latest_version: table.get::<_, String>("latest_version")?,
                    edition: table.get::<_, String>("edition")?,
                    status: DiffStatus::from_str(table.get::<_, String>("status")?, standard)?,
                    diff: {
                        if table.contains_key::<_>("diff")? {
                            Some(DiffInfo::from_table(table.get::<_, LuaTable>("diff")?, standard)?)
                        } else {
                            None
                        }
                    }
                })
            }
        }
    }

    pub fn to_table<'a>(&self, lua: &'a Lua, standard: IntegrationStandard) -> anyhow::Result<LuaTable<'a>> {
        match standard {
            IntegrationStandard::V1 => {
                let table = lua.create_table()?;

                table.set("current_version", self.current_version.clone())?;
                table.set("latest_version", self.latest_version.clone())?;
                table.set("edition", self.edition.clone())?;
                table.set("status", self.status.to_str(standard))?;

                if let Some(diff) = &self.diff {
                    table.set("diff", diff.to_table(lua, standard)?)?;
                }

                Ok(table)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffStatus {
    Latest,
    Outdated,
    Unavailable
}

impl DiffStatus {
    pub fn from_str(value: impl AsRef<str>, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                match value.as_ref() {
                    "latest"      => Ok(Self::Latest),
                    "outdated"    => Ok(Self::Outdated),
                    "unavailable" => Ok(Self::Unavailable),

                    _ => anyhow::bail!("Wrong v1 diff status: '{}'", value.as_ref())
                }
            }
        }
    }

    pub fn to_str(&self, standard: IntegrationStandard) -> &str {
        match standard {
            IntegrationStandard::V1 => {
                match self {
                    Self::Latest      => "latest",
                    Self::Outdated    => "outdated",
                    Self::Unavailable => "unavailable"
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffInfo {
    Archive {
        size: u64,
        uri: String
    },
    Segments {
        size: u64,
        segments: Vec<String>
    },
    Files {
        size: u64,
        files: Vec<String>
    }
}

impl DiffInfo {
    pub fn from_table(table: LuaTable, standard: IntegrationStandard) -> anyhow::Result<Self> {
        match standard {
            IntegrationStandard::V1 => {
                let size = table.get::<_, u64>("size")?;

                match table.get::<_, String>("type")?.as_str() {
                    "archive" => Ok(Self::Archive {
                        size,
                        uri: table.get::<_, String>("uri")?
                    }),

                    "segments" => Ok(Self::Segments {
                        size,
                        segments: table.get::<_, LuaTable>("segments")?
                            .sequence_values::<String>()
                            .flatten()
                            .collect()
                    }),

                    "files" => Ok(Self::Files {
                        size,
                        files: table.get::<_, LuaTable>("files")?
                            .sequence_values::<String>()
                            .flatten()
                            .collect()
                    }),

                    value => anyhow::bail!("Wrong v1 diff type: '{value}'")
                }
            }
        }
    }

    pub fn to_table<'a>(&self, lua: &'a Lua, standard: IntegrationStandard) -> anyhow::Result<LuaTable<'a>> {
        match standard {
            IntegrationStandard::V1 => {
                let table = lua.create_table()?;

                match self {
                    Self::Archive { size, uri } => {
                        table.set("type", "archive")?;
                        table.set("size", *size)?;
                        table.set("uri", uri.clone())?;
                    }

                    Self::Segments { size, segments } => {
                        let segments_lua = lua.create_table()?;

                        for segment in segments {
                            segments_lua.push(segment.clone())?;
                        }

                        table.set("type", "segments")?;
                        table.set("size", *size)?;
                        table.set("segments", segments_lua)?;
                    }

                    Self::Files { size, files } => {
                        let files_lua = lua.create_table()?;

                        for file in files {
                            files_lua.push(file.clone())?;
                        }

                        table.set("type", "files")?;
                        table.set("size", *size)?;
                        table.set("files", files_lua)?;
                    }
                }

                Ok(table)
            }
        }
    }
}
