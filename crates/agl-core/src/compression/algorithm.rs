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

use super::CompressionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionAlgorithm {
    #[cfg(feature = "compression-lz4")]
    Lz4,

    #[cfg(feature = "compression-bzip2")]
    Bzip2,

    #[cfg(feature = "compression-deflate")]
    Deflate,

    #[cfg(feature = "compression-deflate")]
    Gzip,

    #[cfg(feature = "compression-deflate")]
    Zlib,

    #[cfg(feature = "compression-zstd")]
    Zstd
}

impl CompressionAlgorithm {
    pub const fn name(&self) -> &str {
        match self {
            #[cfg(feature = "compression-lz4")]
            Self::Lz4 => "lz4",

            #[cfg(feature = "compression-bzip2")]
            Self::Bzip2 => "bzip2",

            #[cfg(feature = "compression-deflate")]
            Self::Deflate => "deflate",

            #[cfg(feature = "compression-deflate")]
            Self::Gzip => "gzip",

            #[cfg(feature = "compression-deflate")]
            Self::Zlib => "zlib",

            #[cfg(feature = "compression-zstd")]
            Self::Zstd => "zstd"
        }
    }
}

impl std::fmt::Display for CompressionAlgorithm {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl std::str::FromStr for CompressionAlgorithm {
    type Err = CompressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "compression-lz4")]
            "lz4" => Ok(Self::Lz4),

            #[cfg(feature = "compression-bzip2")]
            "bzip2" | "bz2" => Ok(Self::Bzip2),

            #[cfg(feature = "compression-deflate")]
            "deflate" => Ok(Self::Deflate),

            #[cfg(feature = "compression-deflate")]
            "gzip" | "gz" => Ok(Self::Gzip),

            #[cfg(feature = "compression-deflate")]
            "zlib" => Ok(Self::Zlib),

            #[cfg(feature = "compression-zstd")]
            "zstd" => Ok(Self::Zstd),

            _ => Err(CompressionError::UnknownAlgorithm(s.to_string()))
        }
    }
}

impl AsRef<CompressionAlgorithm> for CompressionAlgorithm {
    #[inline(always)]
    fn as_ref(&self) -> &CompressionAlgorithm {
        self
    }
}
