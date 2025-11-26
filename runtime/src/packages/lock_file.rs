use std::collections::HashMap;
use std::str::FromStr;

use serde::{Serialize, Deserialize};
use toml::{Value as Toml, Table as TomlTable};

use crate::hash::{Hash, AsHash};

use super::manifest::{ResourceFormat, PackageManifestError};

#[derive(Debug, thiserror::Error)]
pub enum LockFileError {
    #[error(transparent)]
    PackageManifestError(#[from] PackageManifestError),

    #[error("invalid lock file format version: {0}")]
    InvalidFormatVersion(u16),

    #[error("invalid lock file field '{field}' value: expected '{expected}'")]
    InvalidFieldValue {
        field: &'static str,
        expected: &'static str
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockFile {
    pub lock: LockFileInfo,
    pub resources: Vec<ResourceLock>
}

impl AsHash for LockFile {
    fn hash(&self) -> Hash {
        self.lock.hash()
            .chain(self.resources.hash())
    }

    fn partial_hash(&self) -> Hash {
        self.lock.partial_hash()
            .chain(self.resources.partial_hash())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LockFileInfo {
    pub root: Vec<u32>
}

impl AsHash for LockFileInfo {
    #[inline]
    fn hash(&self) -> Hash {
        self.root.hash()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLock {
    pub url: String,
    pub format: ResourceFormat,
    pub lock: ResourceLockData,
    pub inputs: Option<HashMap<String, u32>>,
    pub outputs: Option<HashMap<String, u32>>
}

impl AsHash for ResourceLock {
    fn hash(&self) -> Hash {
        self.url.hash()
            .chain(self.format.hash())
            .chain(self.lock.hash())
            .chain(self.inputs.hash())
            .chain(self.outputs.hash())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceLockData {
    pub hash: Hash,
    pub size: u64
}

impl AsHash for ResourceLockData {
    #[inline]
    fn hash(&self) -> Hash {
        self.hash
    }
}

impl From<&LockFile> for TomlTable {
    fn from(value: &LockFile) -> Self {
        let mut lock_file = TomlTable::new();

        let mut lock_info = TomlTable::new();

        let root = value.lock.root.iter()
            .map(|id| Toml::Integer(*id as i64))
            .collect::<Vec<_>>();

        lock_info.insert(
            String::from("format"),
            Toml::Integer(1)
        );

        lock_info.insert(
            String::from("root"),
            Toml::Array(root)
        );

        lock_file.insert(
            String::from("lock"),
            Toml::Table(lock_info)
        );

        let resources = value.resources.iter()
            .map(|resource| {
                let mut table = TomlTable::new();

                table.insert(
                    String::from("url"),
                    Toml::String(resource.url.clone())
                );

                table.insert(
                    String::from("format"),
                    Toml::String(resource.format.to_string())
                );

                let mut lock = TomlTable::new();

                lock.insert(
                    String::from("hash"),
                    Toml::String(resource.lock.hash.to_string())
                );

                lock.insert(
                    String::from("size"),
                    Toml::Integer(resource.lock.size as i64)
                );

                table.insert(
                    String::from("lock"),
                    Toml::Table(lock)
                );

                if let Some(inputs) = &resource.inputs {
                    let inputs = inputs.iter()
                        .map(|(name, id)| (name.clone(), Toml::Integer(*id as i64)))
                        .collect::<toml::map::Map<String, Toml>>();

                    table.insert(
                        String::from("inputs"),
                        Toml::Table(inputs)
                    );
                }

                if let Some(outputs) = &resource.outputs {
                    let outputs = outputs.iter()
                        .map(|(name, id)| (name.clone(), Toml::Integer(*id as i64)))
                        .collect::<toml::map::Map<String, Toml>>();

                    table.insert(
                        String::from("outputs"),
                        Toml::Table(outputs)
                    );
                }

                Toml::Table(table)
            })
            .collect::<Vec<_>>();

        lock_file.insert(
            String::from("resources"),
            Toml::Array(resources)
        );

        lock_file
    }
}

impl TryFrom<&TomlTable> for LockFile {
    type Error = LockFileError;

    fn try_from(value: &TomlTable) -> Result<Self, Self::Error> {
        let Some(lock) = value.get("lock").and_then(Toml::as_table) else {
            return Err(LockFileError::InvalidFieldValue {
                field: "lock",
                expected: "table"
            });
        };

        let format = lock.get("format")
            .and_then(Toml::as_integer)
            .ok_or({
                LockFileError::InvalidFieldValue {
                    field: "lock.format",
                    expected: "integer"
                }
            })? as u16;

        match format {
            1 => {
                fn parse_resource(table: &TomlTable) -> Result<ResourceLock, LockFileError> {
                    let url = table.get("url")
                        .and_then(Toml::as_str)
                        .ok_or({
                            LockFileError::InvalidFieldValue {
                                field: "<resource>.url",
                                expected: "string"
                            }
                        })?;

                    let format = table.get("format")
                        .and_then(Toml::as_str)
                        .map(ResourceFormat::from_str)
                        .ok_or({
                            LockFileError::InvalidFieldValue {
                                field: "<resource>.format",
                                expected: "string"
                            }
                        })??;

                    let Some(lock) = table.get("lock").and_then(Toml::as_table) else {
                        return Err(LockFileError::InvalidFieldValue {
                            field: "<resource>.lock",
                            expected: "table"
                        })
                    };

                    let hash = lock.get("hash")
                        .and_then(Toml::as_str)
                        .and_then(Hash::from_base32)
                        .ok_or({
                            LockFileError::InvalidFieldValue {
                                field: "<resource>.lock.hash",
                                expected: "string"
                            }
                        })?;

                    let size = lock.get("size")
                        .and_then(Toml::as_integer)
                        .map(|size| size as u64)
                        .ok_or({
                            LockFileError::InvalidFieldValue {
                                field: "<resource>.lock.size",
                                expected: "integer"
                            }
                        })?;

                    let inputs = table.get("inputs")
                        .and_then(Toml::as_table)
                        .map(|inputs| {
                            inputs.into_iter()
                                .map(|(name, id)| {
                                    id.as_integer()
                                        .map(|id| (name.to_owned(), id as u32))
                                })
                                .collect::<Option<HashMap<String, u32>>>()
                                .ok_or({
                                    LockFileError::InvalidFieldValue {
                                        field: "<resource>.inputs",
                                        expected: "table"
                                    }
                                })
                        })
                        .transpose()?;

                    let outputs = table.get("outputs")
                        .and_then(Toml::as_table)
                        .map(|outputs| {
                            outputs.into_iter()
                                .map(|(name, id)| {
                                    id.as_integer()
                                        .map(|id| (name.to_owned(), id as u32))
                                })
                                .collect::<Option<HashMap<String, u32>>>()
                                .ok_or({
                                    LockFileError::InvalidFieldValue {
                                        field: "<resource>.outputs",
                                        expected: "table"
                                    }
                                })
                        })
                        .transpose()?;

                    Ok(ResourceLock {
                        url: url.to_string(),
                        format,
                        lock: ResourceLockData {
                            hash,
                            size
                        },
                        inputs,
                        outputs
                    })
                }

                let root = lock.get("root")
                    .and_then(Toml::as_array)
                    .and_then(|root| {
                        root.iter()
                            .map(|id| {
                                id.as_integer()
                                    .map(|id| id as u32)
                            })
                            .collect::<Option<Vec<_>>>()
                    })
                    .ok_or({
                        LockFileError::InvalidFieldValue {
                            field: "lock.root",
                            expected: "integer[]"
                        }
                    })?;

                let resources = value.get("resources")
                    .and_then(Toml::as_array)
                    .ok_or({
                        LockFileError::InvalidFieldValue {
                            field: "resources",
                            expected: "array"
                        }
                    })
                    .and_then(|resources| {
                        resources.iter()
                            .map(|resource| {
                                resource.as_table()
                                    .ok_or({
                                        LockFileError::InvalidFieldValue {
                                            field: "resources[]",
                                            expected: "table"
                                        }
                                    })
                                    .and_then(parse_resource)
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })?;

                Ok(Self {
                    lock: LockFileInfo {
                        root
                    },
                    resources
                })
            }

            _ => Err(LockFileError::InvalidFormatVersion(format))
        }
    }
}
