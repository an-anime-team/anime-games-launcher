use serde::{Serialize, Deserialize};

use crate::packages::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Format of the resource archive.
pub enum ResourceArchiveFormat {
    Auto,
    Tar,
    Zip,
    Sevenz
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceFormat {
    Package,
    File,
    Archive(ResourceArchiveFormat)
}

impl ResourceFormat {
    /// Predict resource format from its URI.
    pub fn predict(uri: impl AsRef<str>) -> Self {
        let uri = uri.as_ref()
            .replace('\\', "/");

        // Empty URI tail = package expected.
        let Some(tail) = uri.split('/').last() else {
            return Self::Package;
        };

        if tail.is_empty() || tail == "package.json" {
            Self::Package
        } else if uri.contains(".tar") {
            Self::Archive(ResourceArchiveFormat::Tar)
        } else if tail.ends_with(".zip") {
            Self::Archive(ResourceArchiveFormat::Zip)
        } else if tail.ends_with(".7z") {
            Self::Archive(ResourceArchiveFormat::Sevenz)
        } else {
            Self::File
        }
    }
}

impl std::fmt::Display for ResourceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Package => write!(f, "package"),
            Self::File    => write!(f, "file"),

            Self::Archive(format) => match format {
                ResourceArchiveFormat::Auto   => write!(f, "archive"),
                ResourceArchiveFormat::Tar    => write!(f, "archive/tar"),
                ResourceArchiveFormat::Zip    => write!(f, "archive/zip"),
                ResourceArchiveFormat::Sevenz => write!(f, "archive/7z")
            }
        }
    }
}

impl std::str::FromStr for ResourceFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "package" => Ok(Self::Package),
            "file"    => Ok(Self::File),

            "archive"     => Ok(Self::Archive(ResourceArchiveFormat::Auto)),
            "archive/tar" => Ok(Self::Archive(ResourceArchiveFormat::Tar)),
            "archive/zip" => Ok(Self::Archive(ResourceArchiveFormat::Zip)),
            "archive/7z"  => Ok(Self::Archive(ResourceArchiveFormat::Sevenz)),

            _ => anyhow::bail!("Unsupported resource format: {s}")
        }
    }
}

impl AsHash for ResourceFormat {
    #[inline]
    fn hash(&self) -> Hash {
        self.to_string().hash()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prediction() {
        assert_eq!(ResourceFormat::predict("https://example.org/"), ResourceFormat::Package);
        assert_eq!(ResourceFormat::predict("https://example.org/package.json"), ResourceFormat::Package);
        assert_eq!(ResourceFormat::predict("https://example.org/file"), ResourceFormat::File);
        assert_eq!(ResourceFormat::predict("https://example.org/archive.tar"), ResourceFormat::Archive(ResourceArchiveFormat::Tar));
        assert_eq!(ResourceFormat::predict("https://example.org/archive.zip"), ResourceFormat::Archive(ResourceArchiveFormat::Zip));
        assert_eq!(ResourceFormat::predict("https://example.org/archive.7z"), ResourceFormat::Archive(ResourceArchiveFormat::Sevenz));

        assert_eq!(ResourceFormat::predict("https://example.org/archive.tar.gz"), ResourceFormat::Archive(ResourceArchiveFormat::Tar));
        assert_eq!(ResourceFormat::predict("https://example.org/archive.tar.xz"), ResourceFormat::Archive(ResourceArchiveFormat::Tar));
    }
}
