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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompressionLevel {
    /// 1/5 - very fast, small compression ratio.
    Quick,

    /// 2/5 - fast, small compression ratio.
    Fast,

    /// 3/5 - balanced compression speed and ratio.
    Balanced,

    /// 4/5 - slow, good compression ratio.
    Good,

    /// 5/5 - very slow, good compression ratio.
    Best,

    /// Default native compression level for the selected algorithm.
    #[default]
    Default,

    /// Custom native compression level for the selected algorithm.
    Custom(i8)
}

#[cfg(feature = "compression-zstd")]
impl CompressionLevel {
    /// Convert into zstd compression level.
    pub const fn zstd_level(&self) -> i32 {
        match self {
            Self::Quick    => 3,
            Self::Fast     => 9,
            Self::Balanced => 13,
            Self::Good     => 17,
            Self::Best     => 22,
            Self::Default  => 10,

            Self::Custom(level) => *level as i32
        }
    }
}

impl std::fmt::Display for CompressionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quick    => f.write_str("quick"),
            Self::Fast     => f.write_str("fast"),
            Self::Balanced => f.write_str("balanced"),
            Self::Good     => f.write_str("good"),
            Self::Best     => f.write_str("best"),
            Self::Default  => f.write_str("default"),

            Self::Custom(level) => level.fmt(f)
        }
    }
}

impl std::str::FromStr for CompressionLevel {
    type Err = CompressionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "quick"    => Ok(Self::Quick),
            "fast"     => Ok(Self::Fast),
            "balanced" => Ok(Self::Balanced),
            "good"     => Ok(Self::Good),
            "best"     => Ok(Self::Best),
            "default"  => Ok(Self::Default),

            _ => {
                let Ok(level) = s.parse::<i8>() else {
                    return Err(CompressionError::InvalidLevel(s.to_string()));
                };

                Ok(Self::Custom(level))
            }
        }
    }
}

#[cfg(feature = "compression-bzip2")]
impl From<CompressionLevel> for bzip2::Compression {
    fn from(value: CompressionLevel) -> Self {
        match value {
            CompressionLevel::Quick    => Self::new(1),
            CompressionLevel::Fast     => Self::new(3),
            CompressionLevel::Balanced => Self::new(5),
            CompressionLevel::Good     => Self::new(7),
            CompressionLevel::Best     => Self::new(9),
            CompressionLevel::Default  => Self::new(4),

            CompressionLevel::Custom(level) => Self::new(level as u32)
        }
    }
}

#[cfg(feature = "compression-deflate")]
impl From<CompressionLevel> for flate2::Compression {
    fn from(value: CompressionLevel) -> Self {
        match value {
            CompressionLevel::Quick    => Self::new(1),
            CompressionLevel::Fast     => Self::new(3),
            CompressionLevel::Balanced => Self::new(5),
            CompressionLevel::Good     => Self::new(7),
            CompressionLevel::Best     => Self::new(9),
            CompressionLevel::Default  => Self::new(6),

            CompressionLevel::Custom(level) => Self::new(level as u32)
        }
    }
}

impl AsRef<CompressionLevel> for CompressionLevel {
    #[inline(always)]
    fn as_ref(&self) -> &CompressionLevel {
        self
    }
}

macro_rules! impl_from {
    ($($num:ty)+) => {
        $(
            impl From<$num> for CompressionLevel {
                #[inline(always)]
                fn from(value: $num) -> Self {
                    Self::Custom(value as i8)
                }
            }
        )+
    };
}

impl_from!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);
