use std::path::Path;

const FORMATS: &[(ArchiveFormat, &[&str])] = &[
    (ArchiveFormat::Tar, &[
        ".tar",
        ".tar.xz",
        ".tar.gz",
        ".tar.bz2",
        ".tar.zst",
        ".tar.zstd",
        ".txz",
        ".tgz",
        ".tbz2",
        ".tzst",
        ".tzstd"
    ]),

    (ArchiveFormat::Zip, &[
        ".zip"
    ]),

    (ArchiveFormat::Sevenz, &[
        ".7z",
        ".7z.001",
        ".zip.001"
    ])
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchiveFormat {
    Tar,
    Zip,
    Sevenz
}

impl ArchiveFormat {
    /// Assume archive format from the fs path.
    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        let path = path.as_ref()
            .as_os_str()
            .to_string_lossy();

        for (format, exts) in FORMATS {
            for ext in exts.iter() {
                if path.ends_with(ext) {
                    return Some(*format);
                }
            }
        }

        None
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Self::Tar    => "tar",
            Self::Zip    => "zip",
            Self::Sevenz => "7z"
        }
    }
}

impl std::fmt::Display for ArchiveFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for ArchiveFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tar" => Ok(Self::Tar),
            "zip" => Ok(Self::Zip),
            "7z" | "sevenz" => Ok(Self::Sevenz),

            _ => Err(format!("unsupported format: {s}"))
        }
    }
}
