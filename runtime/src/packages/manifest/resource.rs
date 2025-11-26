use std::str::FromStr;

use serde::{Serialize, Deserialize};

use wineyard_core::archives::ArchiveFormat;

use crate::hash::{Hash, AsHash};

use super::PackageManifestError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceFormat {
    /// Use the file as is, without any special treatment. Allow modules to
    /// read this file.
    File,

    /// Another package which should be imported as a dependency.
    Package,

    /// A script which should be executed in the runtime.
    Module(ResourceModuleFormat),

    /// Extract the files from this archive and allow modules to read them.
    Archive(ResourceArchiveFormat)
}

impl ResourceFormat {
    /// Predict resource format from the URI.
    pub fn from_uri(uri: impl AsRef<str>) -> Self {
        let uri = uri.as_ref()
            .replace('\\', "/")
            .replace("//", "/");

        let (_, file_name) = uri.rsplit_once('/')
            .unwrap_or((&uri, "index.html"));

        if let Some(format) = ArchiveFormat::from_path(file_name) {
            Self::Archive(ResourceArchiveFormat::from(format))
        }

        else if file_name.ends_with(".luau") || file_name.ends_with(".lua") {
            Self::Module(ResourceModuleFormat::Luau)
        }

        else {
            Self::File
        }
    }
}

impl std::fmt::Display for ResourceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File    => f.write_str("file"),
            Self::Package => f.write_str("package"),

            Self::Module(format)  => write!(f, "module/{format}"),
            Self::Archive(format) => write!(f, "archive/{format}")
        }
    }
}

impl FromStr for ResourceFormat {
    type Err = PackageManifestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (primary, secondary) = s.split_once('/')
            .unwrap_or((s, "auto"));

        match primary {
            "file"    => Ok(Self::File),
            "package" => Ok(Self::Package),

            "module" => {
                let secondary = if secondary == "auto" {
                    ResourceModuleFormat::default()
                } else {
                    ResourceModuleFormat::from_str(secondary)?
                };

                Ok(Self::Module(secondary))
            }

            "archive" => {
                let secondary = if secondary == "auto" {
                    ResourceArchiveFormat::default()
                } else {
                    ResourceArchiveFormat::from_str(secondary)?
                };

                Ok(Self::Archive(secondary))
            }

            _ => Err(PackageManifestError::ResourceUnknownFormat(s.to_string()))
        }
    }
}

impl AsHash for ResourceFormat {
    #[inline]
    fn hash(&self) -> Hash {
        Hash::for_slice(self.to_string())
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceModuleFormat {
    #[default]
    Auto,
    Luau
}

impl std::fmt::Display for ResourceModuleFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => f.write_str("auto"),
            Self::Luau => f.write_str("luau")
        }
    }
}

impl FromStr for ResourceModuleFormat {
    type Err = PackageManifestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "luau" | "lua" => Ok(Self::Luau),

            _ => Err(PackageManifestError::ResourceUnknownModuleFormat(s.to_string()))
        }
    }
}

impl AsHash for ResourceModuleFormat {
    #[inline]
    fn hash(&self) -> Hash {
        Hash::for_slice(self.to_string())
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceArchiveFormat {
    #[default]
    Auto,
    Tar,
    Zip,
    Sevenz
}

impl std::fmt::Display for ResourceArchiveFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto   => f.write_str("auto"),
            Self::Tar    => f.write_str("tar"),
            Self::Zip    => f.write_str("zip"),
            Self::Sevenz => f.write_str("7z")
        }
    }
}

impl FromStr for ResourceArchiveFormat {
    type Err = PackageManifestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "tar"  => Ok(Self::Tar),
            "zip"  => Ok(Self::Zip),
            "7z" | "sevenz" => Ok(Self::Sevenz),

            _ => Err(PackageManifestError::ResourceUnknownArchiveFormat(s.to_string()))
        }
    }
}

impl From<ArchiveFormat> for ResourceArchiveFormat {
    fn from(value: ArchiveFormat) -> Self {
        match value {
            ArchiveFormat::Tar    => Self::Tar,
            ArchiveFormat::Zip    => Self::Zip,
            ArchiveFormat::Sevenz => Self::Sevenz
        }
    }
}

impl AsHash for ResourceArchiveFormat {
    #[inline]
    fn hash(&self) -> Hash {
        Hash::for_slice(self.to_string())
    }
}
