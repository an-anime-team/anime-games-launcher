// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-core
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
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

use std::str::FromStr;

/// | Family  | Variant         | Bits | Name              |
/// | ------- | --------------- | ---- | ----------------- |
/// | crc     | crc16/arc       | 16   | `crc16`           |
/// | crc     | crc16/usb       | 16   | `crc16/usb`       |
/// | crc     | crc32/iso-hdlc  | 32   | `crc32`           |
/// | crc     | crc32/iscsi     | 32   | `crc32c`          |
/// | crc     | crc32/bzip2     | 32   | `crc32/bzip2`     |
/// | crc     | crc32/mpeg-2    | 32   | `crc32/mpeg-2`    |
/// | crc     | crc32/cksum     | 32   | `cksum`           |
/// | crc     | crc64/ecma-182  | 64   | `crc64`           |
/// | crc     | crc64/nvme      | 64   | `crc64/nvme`      |
/// | crc     | crc64/xz        | 64   | `crc64/xz`        |
/// | crc     | crc64/westbury  | 64   | `crc64/westbury`  |
/// | crc     | crc64/go-iso    | 64   | `crc64/go-iso`    |
/// | crc     | crc64/microsoft | 64   | `crc64/microsoft` |
/// | crc     | crc64/redis     | 64   | `crc64/redis`     |
/// | seahash | seahash         | 64   | `seahash`         |
/// | siphash | siphash 1-3     | 64   | `siphash-1-3-64`  |
/// | siphash | siphash 1-3     | 128  | `siphash-1-3-128` |
/// | siphash | siphash 2-4     | 64   | `siphash-2-4-64`  |
/// | siphash | siphash 2-4     | 128  | `siphash-2-4-128` |
/// | xxh     | xxh32           | 32   | `xxh-32`          |
/// | xxh     | xxh64           | 64   | `xxh-64`          |
/// | xxh     | xxh3            | 64   | `xxh3-64`         |
/// | xxh     | xxh3            | 128  | `xxh3-128`        |
/// | md      | md5             | 128  | `md5`             |
/// | sha     | sha1            | 160  | `sha1`            |
/// | sha     | sha2            | 224  | `sha2-224`        |
/// | sha     | sha2            | 256  | `sha2-256`        |
/// | sha     | sha2            | 384  | `sha2-384`        |
/// | sha     | sha2            | 512  | `sha2-512`        |
/// | sha     | sha2            | 224  | `sha2-512/224`    |
/// | sha     | sha2            | 256  | `sha2-512/256`    |
/// | sha     | shake           | 128  | `shake-128`       |
/// | sha     | shake           | 256  | `shake-256`       |
/// | sha     | keccak          | 224  | `keccak-224`      |
/// | sha     | keccak          | 256  | `keccak-256`      |
/// | sha     | keccak          | 256  | `keccak-256-full` |
/// | sha     | keccak          | 384  | `keccak-384`      |
/// | sha     | keccak          | 512  | `keccak-512`      |
/// | sha     | sha3            | 224  | `sha3-224`        |
/// | sha     | sha3            | 256  | `sha3-256`        |
/// | sha     | sha3            | 384  | `sha3-384`        |
/// | sha     | sha3            | 512  | `sha3-512`        |
/// | blake   | blake2s         | 128  | `blake2s-128`     |
/// | blake   | blake2s         | 256  | `blake2s`         |
/// | blake   | blake2b         | 128  | `blake2b-128`     |
/// | blake   | blake2b         | 256  | `blake2b-256`     |
/// | blake   | blake2b         | 512  | `blake2b`         |
/// | blake   | blake3          | 256  | `blake3`          |
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    /// CRC-16/ARC (`crc16/arc`, `crc16`)
    ///
    /// This is the classic CRC-16, and most likely what you expect to see in
    /// other applications.
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc16Arc,

    /// CRC-16/USB (`crc16/usb`)
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc16Usb,

    /// CRC-32/ISO-HDLC (`crc32/iso-hdlc`, `crc32/hdlc`, `crc32`)
    ///
    /// This is the classic CRC-32, and most likely what you expect to see in
    /// other applications.
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc32IsoHdlc,

    /// CRC-32/ISCSI (`crc32/iscsi`, `crc32c`)
    ///
    /// This is a very common variant of classic CRC-32.
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc32Iscsi,

    /// CRC-32/BZIP2 (`crc32/bzip2`, `crc32/bz2`)
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc32Bzip2,

    /// CRC-32/MPEG-2 (`crc32/mpeg-2`, `crc32/mpeg2`)
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc32Mpeg2,

    /// CRC-32/CKSUM (`crc32/cksum`, `crc32/posix`, `cksum`)
    ///
    /// This is a variant of CRC-32 used in POSIX `cksum` command.
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc32Cksum,

    /// CRC-64/ECMA-182 (`crc64/ecma-182`, `crc64/ecma182`, `crc64`)
    ///
    /// This is the classic CRC-64, and most likely what you expect to see in
    /// other applications.
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc64Ecma182,

    /// CRC-64/NVME (`crc64/nvme`)
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc64Nvme,

    /// CRC-64/XZ (`crc64/xz`)
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc64Xz,

    /// CRC-64/WE (`crc64/westbury`, `crc64/we`)
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc64Westbury,

    /// CRC-64/GO-ISO (`crc64/go-iso`, `crc64/go`)
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc64GoIso,

    /// CRC-64/MS (`crc64/microsoft`, `crc64/ms`)
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc64Microsoft,

    /// CRC-64/REDIS (`crc64/redis`)
    ///
    /// Source: https://reveng.sourceforge.io/crc-catalogue/all.htm
    #[cfg(feature = "hashes-crc")]
    Crc64Redis,

    /// Seahash (`seahash`)
    #[cfg(feature = "hashes-seahash")]
    Seahash,

    /// Siphash 1-3 64 bit (`siphash-1-3-64`)
    #[cfg(feature = "hashes-siphash")]
    Siphash_1_3_64,

    /// Siphash 1-3 128 bit (`siphash-1-3-128`)
    #[cfg(feature = "hashes-siphash")]
    Siphash_1_3_128,

    /// Siphash 2-4 64 bit (`siphash-1-3-64`)
    #[cfg(feature = "hashes-siphash")]
    Siphash_2_4_64,

    /// Siphash 2-4 128 bit (`siphash-1-3-128`)
    #[cfg(feature = "hashes-siphash")]
    Siphash_2_4_128,

    /// XXH32 (`xxh32`)
    #[cfg(feature = "hashes-xxh")]
    Xxh_32,

    /// XXH64 (`xxh64`)
    #[cfg(feature = "hashes-xxh")]
    Xxh_64,

    /// XXH3 64 bit (`xxh3-64`)
    #[cfg(feature = "hashes-xxh")]
    Xxh3_64,

    /// XXH3 128 bit (`xxh3-128`)
    #[cfg(feature = "hashes-xxh")]
    Xxh3_128,

    /// MD5 (`md5`)
    #[cfg(feature = "hashes-md5")]
    Md5,

    /// SHA1 (`sha1`)
    #[cfg(feature = "hashes-sha1")]
    Sha1,

    #[cfg(feature = "hashes-sha2")]
    Sha2_224,

    #[cfg(feature = "hashes-sha2")]
    Sha2_256,

    #[cfg(feature = "hashes-sha2")]
    Sha2_384,

    #[cfg(feature = "hashes-sha2")]
    Sha2_512,

    #[cfg(feature = "hashes-sha2")]
    Sha2_512_224,

    #[cfg(feature = "hashes-sha2")]
    Sha2_512_256,

    #[cfg(feature = "hashes-sha3")]
    Shake_128,

    #[cfg(feature = "hashes-sha3")]
    Shake_256,

    #[cfg(feature = "hashes-sha3")]
    Keccak_224,

    #[cfg(feature = "hashes-sha3")]
    Keccak_256,

    #[cfg(feature = "hashes-sha3")]
    Keccak_256_Full,

    #[cfg(feature = "hashes-sha3")]
    Keccak_384,

    #[cfg(feature = "hashes-sha3")]
    Keccak_512,

    #[cfg(feature = "hashes-sha3")]
    Sha3_224,

    #[cfg(feature = "hashes-sha3")]
    Sha3_256,

    #[cfg(feature = "hashes-sha3")]
    Sha3_384,

    #[cfg(feature = "hashes-sha3")]
    Sha3_512,

    /// Blake2s 128 bit (`blake2s-128`)
    #[cfg(feature = "hashes-blake2")]
    Blake2s_128,

    /// Blake2s 256 bit (`blake2s`, `blake2s-256`)
    ///
    /// This is the standard blake2s variant.
    #[cfg(feature = "hashes-blake2")]
    Blake2s_256,

    /// Blake2b 128 bit (`blake2b-128`)
    #[cfg(feature = "hashes-blake2")]
    Blake2b_128,

    /// Blake2b 256 bit (`blake2b-256`)
    #[cfg(feature = "hashes-blake2")]
    Blake2b_256,

    /// Blake2b 512 bit (`blake2b`, `blake2b-512`)
    ///
    /// This is the standard blake2b variant.
    #[cfg(feature = "hashes-blake2")]
    Blake2b_512,

    /// Blake3 (`blake3`)
    #[cfg(feature = "hashes-blake3")]
    Blake3
}

impl HashAlgorithm {
    pub const fn name(&self) -> &'static str {
        match self {
            #[cfg(feature = "hashes-crc")]
            Self::Crc16Arc => "crc16/arc",

            #[cfg(feature = "hashes-crc")]
            Self::Crc16Usb => "crc16/usb",

            #[cfg(feature = "hashes-crc")]
            Self::Crc32IsoHdlc => "crc32/iso-hdlc",

            #[cfg(feature = "hashes-crc")]
            Self::Crc32Iscsi => "crc32/iscsi",

            #[cfg(feature = "hashes-crc")]
            Self::Crc32Bzip2 => "crc32/bzip2",

            #[cfg(feature = "hashes-crc")]
            Self::Crc32Mpeg2 => "crc32/mpeg-2",

            #[cfg(feature = "hashes-crc")]
            Self::Crc32Cksum => "crc32/cksum",

            #[cfg(feature = "hashes-crc")]
            Self::Crc64Ecma182 => "crc64/ecma-182",

            #[cfg(feature = "hashes-crc")]
            Self::Crc64Nvme => "crc64/nvme",

            #[cfg(feature = "hashes-crc")]
            Self::Crc64Xz => "crc64/xz",

            #[cfg(feature = "hashes-crc")]
            Self::Crc64Westbury => "crc64/westbury",

            #[cfg(feature = "hashes-crc")]
            Self::Crc64GoIso => "crc64/go-iso",

            #[cfg(feature = "hashes-crc")]
            Self::Crc64Microsoft => "crc64/microsoft",

            #[cfg(feature = "hashes-crc")]
            Self::Crc64Redis => "crc64/redis",

            #[cfg(feature = "hashes-seahash")]
            Self::Seahash => "seahash",

            #[cfg(feature = "hashes-siphash")]
            Self::Siphash_1_3_64 => "siphash-1-3-64",

            #[cfg(feature = "hashes-siphash")]
            Self::Siphash_1_3_128 => "siphash-1-3-128",

            #[cfg(feature = "hashes-siphash")]
            Self::Siphash_2_4_64 => "siphash-2-4-64",

            #[cfg(feature = "hashes-siphash")]
            Self::Siphash_2_4_128 => "siphash-2-4-128",

            #[cfg(feature = "hashes-xxh")]
            Self::Xxh_32 => "xxh-32",

            #[cfg(feature = "hashes-xxh")]
            Self::Xxh_64 => "xxh-64",

            #[cfg(feature = "hashes-xxh")]
            Self::Xxh3_64 => "xxh3-64",

            #[cfg(feature = "hashes-xxh")]
            Self::Xxh3_128 => "xxh3-128",

            #[cfg(feature = "hashes-md5")]
            Self::Md5 => "md5",

            #[cfg(feature = "hashes-sha1")]
            Self::Sha1 => "sha1",

            #[cfg(feature = "hashes-sha2")]
            Self::Sha2_224 => "sha2-224",

            #[cfg(feature = "hashes-sha2")]
            Self::Sha2_256 => "sha2-256",

            #[cfg(feature = "hashes-sha2")]
            Self::Sha2_384 => "sha2-384",

            #[cfg(feature = "hashes-sha2")]
            Self::Sha2_512 => "sha2-512",

            #[cfg(feature = "hashes-sha2")]
            Self::Sha2_512_224 => "sha2-512/224",

            #[cfg(feature = "hashes-sha2")]
            Self::Sha2_512_256 => "sha2-512/256",

            #[cfg(feature = "hashes-sha3")]
            Self::Shake_128 => "shake-128",

            #[cfg(feature = "hashes-sha3")]
            Self::Shake_256 => "shake-256",

            #[cfg(feature = "hashes-sha3")]
            Self::Keccak_224 => "keccak-224",

            #[cfg(feature = "hashes-sha3")]
            Self::Keccak_256 => "keccak-256",

            #[cfg(feature = "hashes-sha3")]
            Self::Keccak_256_Full => "keccak-256-full",

            #[cfg(feature = "hashes-sha3")]
            Self::Keccak_384 => "keccak-384",

            #[cfg(feature = "hashes-sha3")]
            Self::Keccak_512 => "keccak-512",

            #[cfg(feature = "hashes-sha3")]
            Self::Sha3_224 => "sha3-224",

            #[cfg(feature = "hashes-sha3")]
            Self::Sha3_256 => "sha3-256",

            #[cfg(feature = "hashes-sha3")]
            Self::Sha3_384 => "sha3-384",

            #[cfg(feature = "hashes-sha3")]
            Self::Sha3_512 => "sha3-512",

            #[cfg(feature = "hashes-blake2")]
            Self::Blake2s_128 => "blake2s-128",

            #[cfg(feature = "hashes-blake2")]
            Self::Blake2s_256 => "blake2s-256",

            #[cfg(feature = "hashes-blake2")]
            Self::Blake2b_128 => "blake2b-128",

            #[cfg(feature = "hashes-blake2")]
            Self::Blake2b_256 => "blake2b-256",

            #[cfg(feature = "hashes-blake2")]
            Self::Blake2b_512 => "blake2b-512",

            #[cfg(feature = "hashes-blake2")]
            Self::Blake3 => "blake3"
        }
    }
}

impl FromStr for HashAlgorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            #[cfg(feature = "hashes-crc")]
            "crc16/arc" | "crc16" => Ok(Self::Crc16Arc),

            #[cfg(feature = "hashes-crc")]
            "crc16/usb" => Ok(Self::Crc16Usb),

            #[cfg(feature = "hashes-crc")]
            "crc32/iso-hdlc" | "crc32/hdlc" | "crc32" => Ok(Self::Crc32IsoHdlc),

            #[cfg(feature = "hashes-crc")]
            "crc32/iscsi" | "crc32c" => Ok(Self::Crc32Iscsi),

            #[cfg(feature = "hashes-crc")]
            "crc32/bzip2" | "crc32/bzip-2" | "crc32/bz2" => Ok(Self::Crc32Bzip2),

            #[cfg(feature = "hashes-crc")]
            "crc32/mpeg-2" | "crc32/mpeg2" => Ok(Self::Crc32Mpeg2),

            #[cfg(feature = "hashes-crc")]
            "crc32/cksum" | "crc32/posix" | "cksum" => Ok(Self::Crc32Cksum),

            #[cfg(feature = "hashes-crc")]
            "crc64/ecma-182" | "crc64/ecma182" | "crc64" => Ok(Self::Crc64Ecma182),

            #[cfg(feature = "hashes-crc")]
            "crc64/nvme" => Ok(Self::Crc64Nvme),

            #[cfg(feature = "hashes-crc")]
            "crc64/xz" => Ok(Self::Crc64Xz),

            #[cfg(feature = "hashes-crc")]
            "crc64/westbury" | "crc64/we" => Ok(Self::Crc64Westbury),

            #[cfg(feature = "hashes-crc")]
            "crc64/go-iso" | "crc64/go" => Ok(Self::Crc64GoIso),

            #[cfg(feature = "hashes-crc")]
            "crc64/microsoft" | "crc64/ms" => Ok(Self::Crc64Microsoft),

            #[cfg(feature = "hashes-crc")]
            "crc64/redis" => Ok(Self::Crc64Redis),

            #[cfg(feature = "hashes-seahash")]
            "seahash" => Ok(Self::Seahash),

            #[cfg(feature = "hashes-siphash")]
            "siphash-1-3-64" | "siphash-1-3" => Ok(Self::Siphash_1_3_64),

            #[cfg(feature = "hashes-siphash")]
            "siphash-1-3-128" => Ok(Self::Siphash_1_3_128),

            #[cfg(feature = "hashes-siphash")]
            "siphash-2-4-64" | "siphash-2-4" | "siphash" => Ok(Self::Siphash_2_4_64),

            #[cfg(feature = "hashes-siphash")]
            "siphash-2-4-128" => Ok(Self::Siphash_2_4_128),

            #[cfg(feature = "hashes-xxh")]
            "xxh-32" | "xxh32" => Ok(Self::Xxh_32),

            #[cfg(feature = "hashes-xxh")]
            "xxh-64" | "xxh64" => Ok(Self::Xxh_64),

            #[cfg(feature = "hashes-xxh")]
            "xxh3-64" | "xxh3" => Ok(Self::Xxh3_64),

            #[cfg(feature = "hashes-xxh")]
            "xxh3-128" => Ok(Self::Xxh3_128),

            #[cfg(feature = "hashes-md5")]
            "md5" => Ok(Self::Md5),

            #[cfg(feature = "hashes-sha1")]
            "sha1" => Ok(Self::Sha1),

            #[cfg(feature = "hashes-sha2")]
            "sha2-224" => Ok(Self::Sha2_224),

            #[cfg(feature = "hashes-sha2")]
            "sha2-256" | "sha2" => Ok(Self::Sha2_256),

            #[cfg(feature = "hashes-sha2")]
            "sha2-384" => Ok(Self::Sha2_384),

            #[cfg(feature = "hashes-sha2")]
            "sha2-512" => Ok(Self::Sha2_512),

            #[cfg(feature = "hashes-sha2")]
            "sha2-512/224" | "sha2-512-224" => Ok(Self::Sha2_512_224),

            #[cfg(feature = "hashes-sha2")]
            "sha2-512/256" | "sha2-512-256" => Ok(Self::Sha2_512_256),

            #[cfg(feature = "hashes-sha3")]
            "shake-128" | "shake128" => Ok(Self::Shake_128),

            #[cfg(feature = "hashes-sha3")]
            "shake-256" | "shake256" => Ok(Self::Shake_256),

            #[cfg(feature = "hashes-sha3")]
            "keccak-224" | "keccak224" => Ok(Self::Keccak_224),

            #[cfg(feature = "hashes-sha3")]
            "keccak-256" | "keccak256" => Ok(Self::Keccak_256),

            #[cfg(feature = "hashes-sha3")]
            "keccak-256-full" | "keccak256-full" | "keccak256full" => Ok(Self::Keccak_256_Full),

            #[cfg(feature = "hashes-sha3")]
            "keccak-384" | "keccak384" => Ok(Self::Keccak_384),

            #[cfg(feature = "hashes-sha3")]
            "keccak-512" | "keccak512" => Ok(Self::Keccak_512),

            #[cfg(feature = "hashes-sha3")]
            "sha3-224" => Ok(Self::Sha3_224),

            #[cfg(feature = "hashes-sha3")]
            "sha3-256" => Ok(Self::Sha3_256),

            #[cfg(feature = "hashes-sha3")]
            "sha3-384" => Ok(Self::Sha3_384),

            #[cfg(feature = "hashes-sha3")]
            "sha3-512" => Ok(Self::Sha3_512),

            #[cfg(feature = "hashes-blake2")]
            "blake2s-128" => Ok(Self::Blake2s_128),

            #[cfg(feature = "hashes-blake2")]
            "blake2s-256" | "blake2s" => Ok(Self::Blake2s_256),

            #[cfg(feature = "hashes-blake2")]
            "blake2b-128" => Ok(Self::Blake2b_128),

            #[cfg(feature = "hashes-blake2")]
            "blake2b-256" => Ok(Self::Blake2b_256),

            #[cfg(feature = "hashes-blake2")]
            "blake2b-512" | "blake2b" => Ok(Self::Blake2b_512),

            #[cfg(feature = "hashes-blake3")]
            "blake3" => Ok(Self::Blake3),

            _ => Err(format!("unsupported hash algorithm: {s}"))
        }
    }
}

impl std::fmt::Display for HashAlgorithm {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl AsRef<HashAlgorithm> for HashAlgorithm {
    #[inline(always)]
    fn as_ref(&self) -> &HashAlgorithm {
        self
    }
}
