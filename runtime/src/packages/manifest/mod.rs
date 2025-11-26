use std::collections::HashMap;
use std::str::FromStr;

use serde::{Serialize, Deserialize};
use toml::{Table as TomlTable, Value as Toml};

use crate::hash::{Hash, AsHash};

mod error;
mod resource;

pub use error::*;
pub use resource::*;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Information about the current package (metadata fields).
    pub package: PackageInfo,

    /// Requirements for the modules runtime.
    pub runtime: RuntimeInfo,

    /// Table of resources which will be imported and available for the current
    /// package (all the outputs of the current package).
    pub inputs: HashMap<String, ResourceInfo>,

    /// Table of resources which will be exported from the current package and
    /// will be available to other packages.
    pub outputs: HashMap<String, ResourceInfo>
}

impl AsHash for PackageManifest {
    fn hash(&self) -> Hash {
        self.package.hash()
            .chain(self.inputs.hash())
            .chain(self.outputs.hash())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Description of the package.
    pub description: Option<String>,

    /// List of the package's authors.
    pub authors: Vec<String>
}

impl PackageInfo {
    pub fn is_empty(&self) -> bool {
        self.description.is_none() && self.authors.is_empty()
    }
}

impl AsHash for PackageInfo {
    fn hash(&self) -> Hash {
        self.description.hash()
            .chain(self.authors.hash())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeInfo {
    /// Minimal required version of the modules runtime.
    pub minimal_version: u32
}

impl RuntimeInfo {
    pub const fn is_empty(&self) -> bool {
        self.minimal_version < 2
    }
}

impl AsHash for RuntimeInfo {
    #[inline]
    fn hash(&self) -> Hash {
        self.minimal_version.hash()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// Relative or absolute path to the resource.
    pub uri: String,

    /// Format of the resource. Automatically determined from the URI if not
    /// specified explicitly.
    pub format: ResourceFormat,

    /// Hash of the resource. If specified and is not matched at validation time
    /// then the package will not be accepted.
    pub hash: Option<Hash>
}

impl AsHash for ResourceInfo {
    fn hash(&self) -> Hash {
        Hash::for_slice(&self.uri)
            .chain(self.format.hash())
            .chain(self.hash.hash())
    }
}

impl From<&PackageManifest> for TomlTable {
    fn from(value: &PackageManifest) -> Self {
        let mut manifest = TomlTable::new();

        let mut package = TomlTable::new();

        package.insert(
            String::from("format"),
            Toml::Integer(1)
        );

        if let Some(description) = &value.package.description {
            package.insert(
                String::from("description"),
                Toml::String(description.clone())
            );
        }

        if !value.package.authors.is_empty() {
            let authors = value.package.authors
                .iter()
                .cloned()
                .map(Toml::String)
                .collect();

            package.insert(
                String::from("authors"),
                Toml::Array(authors)
            );
        }

        manifest.insert(
            String::from("package"),
            Toml::Table(package)
        );

        if !value.runtime.is_empty() {
            let mut runtime = TomlTable::new();

            runtime.insert(
                String::from("minimal_version"),
                Toml::Integer(value.runtime.minimal_version as i64)
            );

            manifest.insert(
                String::from("runtime"),
                Toml::Table(runtime)
            );
        }

        fn encode_resource(resource: &ResourceInfo) -> TomlTable {
            let mut table = TomlTable::new();

            table.insert(
                String::from("uri"),
                Toml::String(resource.uri.clone())
            );

            table.insert(
                String::from("format"),
                Toml::String(resource.format.to_string())
            );

            if let Some(hash) = resource.hash {
                table.insert(
                    String::from("hash"),
                    Toml::String(hash.to_base32())
                );
            }

            table
        }

        if !value.inputs.is_empty() {
            let mut inputs = TomlTable::new();

            for (name, resource) in &value.inputs {
                inputs.insert(
                    name.clone(),
                    Toml::Table(encode_resource(resource))
                );
            }

            manifest.insert(
                String::from("inputs"),
                Toml::Table(inputs)
            );
        }

        if !value.outputs.is_empty() {
            let mut outputs = TomlTable::new();

            for (name, resource) in &value.outputs {
                outputs.insert(
                    name.clone(),
                    Toml::Table(encode_resource(resource))
                );
            }

            manifest.insert(
                String::from("outputs"),
                Toml::Table(outputs)
            );
        }

        manifest
    }
}

impl TryFrom<&TomlTable> for PackageManifest {
    type Error = PackageManifestError;

    fn try_from(value: &TomlTable) -> Result<Self, Self::Error> {
        let mut manifest = Self::default();

        let Some(package) = value.get("package").and_then(Toml::as_table) else {
            return Err(PackageManifestError::PackageInvalidFieldFormat {
                field: "package",
                expected: "table"
            });
        };

        let format = package.get("format")
            .and_then(Toml::as_integer)
            .ok_or({
                PackageManifestError::PackageInvalidFieldFormat {
                    field: "package.format",
                    expected: "integer"
                }
            })? as u16;

        match format {
            1 => {
                if let Some(description) = package.get("description") {
                    let Some(description) = description.as_str() else {
                        return Err(PackageManifestError::PackageInvalidFieldFormat {
                            field: "package.description",
                            expected: "string"
                        });
                    };

                    manifest.package.description = Some(description.to_string());
                }

                if let Some(authors) = package.get("authors") {
                    let Some(authors) = authors.as_array() else {
                        return Err(PackageManifestError::PackageInvalidFieldFormat {
                            field: "package.authors",
                            expected: "string[]"
                        });
                    };

                    manifest.package.authors = authors.iter()
                        .map(|author| author.as_str().map(String::from))
                        .collect::<Option<Vec<String>>>()
                        .ok_or({
                            PackageManifestError::PackageInvalidFieldFormat {
                                field: "package.authors",
                                expected: "string[]"
                            }
                        })?;
                }

                if let Some(runtime) = value.get("runtime") {
                    let Some(runtime) = runtime.as_table() else {
                        return Err(PackageManifestError::PackageInvalidFieldFormat {
                            field: "runtime",
                            expected: "table"
                        });
                    };

                    if let Some(minimal_version) = runtime.get("minimal_version") {
                        let Some(minimal_version) = minimal_version.as_integer() else {
                            return Err(PackageManifestError::PackageInvalidFieldFormat {
                                field: "runtime.minimal_version",
                                expected: "integer"
                            });
                        };

                        manifest.runtime.minimal_version = minimal_version as u32;
                    }
                }

                fn parse_resource(
                    resource: &TomlTable
                ) -> Result<ResourceInfo, PackageManifestError> {
                    let Some(uri) = resource.get("uri") else {
                        return Err(PackageManifestError::ResourceMissingUri);
                    };

                    let Some(uri) = uri.as_str() else {
                        return Err(PackageManifestError::PackageInvalidFieldFormat {
                            field: "<resource>.uri",
                            expected: "string"
                        });
                    };

                    let format = match resource.get("format") {
                        Some(format) => {
                            let Some(format) = format.as_str() else {
                                return Err(PackageManifestError::PackageInvalidFieldFormat {
                                    field: "<resource>.format",
                                    expected: "string"
                                });
                            };

                            ResourceFormat::from_str(format)?
                        }

                        None => ResourceFormat::from_uri(uri)
                    };

                    let hash = resource.get("hash")
                        .map(|hash| {
                            if let Some(hash) = hash.as_str() {
                                Hash::from_base32(hash)
                                    .ok_or({
                                        PackageManifestError::PackageInvalidFieldFormat {
                                            field: "<resource>.hash",
                                            expected: "string"
                                        }
                                    })
                            }

                            else if let Some(hash) = hash.as_integer() {
                                Ok(Hash(hash as u64))
                            }

                            else {
                                Err(PackageManifestError::PackageInvalidFieldFormat {
                                    field: "<resource>.hash",
                                    expected: "string"
                                })
                            }
                        })
                        .transpose()?;

                    Ok(ResourceInfo {
                        uri: uri.to_string(),
                        format,
                        hash
                    })
                }

                if let Some(inputs) = value.get("inputs") {
                    let Some(inputs) = inputs.as_table() else {
                        return Err(PackageManifestError::PackageInvalidFieldFormat {
                            field: "inputs",
                            expected: "array"
                        });
                    };

                    manifest.inputs = inputs.iter()
                        .map(|(name, resource)| {
                            if let Some(uri) = resource.as_str() {
                                Ok((name.to_owned(), ResourceInfo {
                                    uri: uri.to_string(),
                                    format: ResourceFormat::from_uri(uri),
                                    hash: None
                                }))
                            }

                            else if let Some(resource) = resource.as_table() {
                                parse_resource(resource)
                                    .map(|resource| (name.to_owned(), resource))
                            }

                            else {
                                Err(PackageManifestError::PackageInvalidFieldFormat {
                                    field: "inputs[]",
                                    expected: "table"
                                })
                            }
                        })
                        .collect::<Result<HashMap<_, _>, _>>()?;
                }

                if let Some(outputs) = value.get("outputs") {
                    let Some(outputs) = outputs.as_table() else {
                        return Err(PackageManifestError::PackageInvalidFieldFormat {
                            field: "outputs",
                            expected: "array"
                        });
                    };

                    manifest.outputs = outputs.iter()
                        .map(|(name, resource)| {
                            if let Some(uri) = resource.as_str() {
                                Ok((name.to_owned(), ResourceInfo {
                                    uri: uri.to_string(),
                                    format: ResourceFormat::from_uri(uri),
                                    hash: None
                                }))
                            }

                            else if let Some(resource) = resource.as_table() {
                                parse_resource(resource)
                                    .map(|resource| (name.to_owned(), resource))
                            }

                            else {
                                Err(PackageManifestError::PackageInvalidFieldFormat {
                                    field: "outputs[]",
                                    expected: "table"
                                })
                            }
                        })
                        .collect::<Result<HashMap<_, _>, _>>()?;
                }
            }

            _ => return Err(PackageManifestError::PackageUnknownFormatVersion(format))
        }

        Ok(manifest)
    }
}
