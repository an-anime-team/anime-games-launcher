use std::collections::HashMap;

use super::hash::Hash;

mod v1;
mod v2;

pub use v1::parse_v1;
pub use v2::parse_v2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    /// Version of the manifest file format
    pub manifest_version: u64,

    /// Manifest's metadata (maintainers, homepage)
    pub metadata: ManifestMetadata,

    /// Inputs of this package
    pub inputs: HashMap<String, ManifestInput>,

    /// Outputs from this package
    pub outputs: HashMap<String, ManifestOutput>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Manifest's metadata
pub struct ManifestMetadata {
    /// Address to the manifest's home page (project repository)
    pub homepage: Option<String>,

    /// List of package maintainers
    /// 
    /// Example:
    /// 
    /// - `@johndoe https://john-doe.com`
    /// - `John Doe <john@doe.com>`
    pub maintainers: Option<Vec<String>>
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestInput {
    /// Input's format
    pub format: ManifestInputFormat,

    /// Expected input's hash
    /// 
    /// Read about hash formats in the `ManifestOutput` docs
    pub hash: Hash,

    /// URI to the input file
    pub uri: String
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestOutput {
    /// Output's format
    /// 
    /// Optional field. By default equals to `OutputFormat::Package`
    pub format: ManifestOutputFormat,

    /// Announced output's hash
    /// 
    /// This value is used to compare with the downloaded
    /// package. If it's different from what launcher has,
    /// then this output will be downloaded. Afterwards
    /// launcher will calculate its hash again and compare
    /// with this announced value. If this value will be
    /// different from what launcher got - it will fail
    /// to load this package and display an error message
    /// 
    /// Hash is a base32 encoded value following
    /// [RFC4648 - Base 32 Encoding with Extended Hex Alphabet](https://www.rfc-editor.org/rfc/rfc4648#page-10)
    /// without padding
    /// 
    /// We're using 64 bits variant of [xxh3](https://github.com/Cyan4973/xxHash)
    /// to hash all the files
    pub hash: Hash,

    /// Relative path to the output's entry point
    /// 
    /// Must be a path to existing lua script - either
    /// package or integration script's entry
    pub path: String,

    /// Output's metadata (name, title, etc.)
    pub metadata: ManifestOutputMetadata
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestOutputMetadata {
    /// Output's title
    /// 
    /// Mostly needed for game integration scripts
    pub title: Option<String>,

    /// Integration script standard
    pub standard: Option<u64>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Input package format
pub enum ManifestInputFormat {
    /// Depend on another package
    Package,

    /// Depend on an arbitrary file
    /// 
    /// This file will be downloaded if its announced hash
    /// is different from what is cached by the launcher
    File,

    /// Depend on a ZIP archive
    /// 
    /// This file will be downloaded and extracted if its announced hash
    /// is different from what is cached by the launcher
    /// 
    /// Hash is calculated from the extracted archive's content using
    /// a special custom algorithm. Package developers can keep hash
    /// field empty and look for a calculated hash in the launcher logs
    ZipArchive,

    /// Depend on a TAR archive
    /// 
    /// This file will be downloaded and extracted if its announced hash
    /// is different from what is cached by the launcher
    /// 
    /// Hash is calculated from the extracted archive's content using
    /// a special custom algorithm. Package developers can keep hash
    /// field empty and look for a calculated hash in the launcher logs
    TarArchive,

    /// Depend on a GZ-compressed TAR archive
    /// 
    /// Read `ManifestInputFormat::TarArchive` for details
    TarGzArchive
}

impl Default for ManifestInputFormat {
    #[inline]
    fn default() -> Self {
        Self::File
    }
}

impl ManifestInputFormat {
    #[inline]
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Package      => "package",
            Self::File         => "file",
            Self::ZipArchive   => "zip",
            Self::TarArchive   => "tar",
            Self::TarGzArchive => "targz"
        }
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: impl AsRef<str>) -> Option<Self> {
        match str.as_ref() {
            "package" => Some(Self::Package),
            "file"    => Some(Self::File),
            "zip"     => Some(Self::ZipArchive),
            "tar"     => Some(Self::TarArchive),
            "targz"   => Some(Self::TarGzArchive),

            _ => None
        }
    }

    #[inline]
    /// Check if current input format is a package
    pub fn is_package(&self) -> bool {
        self == &Self::Package
    }

    #[inline]
    /// Check if current input format is an archive
    pub fn is_archive(&self) -> bool {
        [Self::ZipArchive, Self::TarArchive, Self::TarGzArchive].contains(self)
    }

    #[inline]
    /// Check if current input format is a file
    pub fn is_file(&self) -> bool {
        self == &Self::File
    }

    /// Try to identify input format based on its URI
    /// 
    /// Can give false results
    pub fn from_uri(uri: impl AsRef<str>) -> Self {
        let uri = uri.as_ref();

        if uri.ends_with(".lua") {
            Self::Package
        }

        else if uri.ends_with(".zip") {
            Self::ZipArchive
        }

        else if uri.ends_with(".tar.gz") {
            Self::TarGzArchive
        }

        else if uri.ends_with(".tar") {
            Self::TarArchive
        }

        else {
            Self::File
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Output format
pub enum ManifestOutputFormat {
    /// Allow other packages and integration scripts
    /// to access this output's content using
    /// special API methods
    Package,

    /// Game integration script that should be loaded
    /// directly by the launcher
    Integration
}

impl Default for ManifestOutputFormat {
    #[inline]
    fn default() -> Self {
        Self::Package
    }
}

impl ManifestOutputFormat {
    #[inline]
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Package     => "package",
            Self::Integration => "integration"
        }
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: impl AsRef<str>) -> Option<Self> {
        match str.as_ref() {
            "package"     => Some(Self::Package),
            "integration" => Some(Self::Integration),

            _ => None
        }
    }
}
