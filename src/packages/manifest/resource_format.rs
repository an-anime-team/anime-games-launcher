use serde::{Serialize, Deserialize};

use crate::packages::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceFormat {
    Package,
    File,
    Archive,
    Tar,
    Zip,
    Sevenz
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
            Self::Tar
        } else if tail.ends_with(".zip") {
            Self::Zip
        } else if tail.ends_with(".7z") {
            Self::Sevenz
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
            Self::Archive => write!(f, "archive"),
            Self::Tar     => write!(f, "tar"),
            Self::Zip     => write!(f, "zip"),
            Self::Sevenz  => write!(f, "7z")
        }
    }
}

impl std::str::FromStr for ResourceFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "package" => Ok(Self::Package),
            "file"    => Ok(Self::File),
            "archive" => Ok(Self::Archive),
            "tar"     => Ok(Self::Tar),
            "zip"     => Ok(Self::Zip),
            "7z"      => Ok(Self::Sevenz),

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
        assert_eq!(ResourceFormat::predict("https://example.org/archive.tar"), ResourceFormat::Tar);
        assert_eq!(ResourceFormat::predict("https://example.org/archive.zip"), ResourceFormat::Zip);
        assert_eq!(ResourceFormat::predict("https://example.org/archive.7z"), ResourceFormat::Sevenz);

        assert_eq!(ResourceFormat::predict("https://example.org/archive.tar.gz"), ResourceFormat::Tar);
        assert_eq!(ResourceFormat::predict("https://example.org/archive.tar.xz"), ResourceFormat::Tar);
    }
}
