// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-core
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
    /// Predict archive format from its filename.
    pub fn from_filename(name: impl AsRef<str>) -> Option<Self> {
        let name = name.as_ref();

        for (format, exts) in FORMATS {
            for ext in exts.iter() {
                if name.ends_with(ext) {
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

impl AsRef<ArchiveFormat> for ArchiveFormat {
    #[inline(always)]
    fn as_ref(&self) -> &ArchiveFormat {
        self
    }
}
