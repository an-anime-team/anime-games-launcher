use std::str::FromStr;

/// | Family  | Variant     | Bits | Name              |
/// | ------- | ----------- | ---- | ----------------- |
/// | seahash | seahash     | 64   | `seahash`         |
/// | crc     | crc32       | 32   | `crc32`           |
/// | crc     | crc32c      | 32   | `crc32c`          |
/// | siphash | siphash 1-3 | 64   | `siphash-1-3-64`  |
/// | siphash | siphash 1-3 | 128  | `siphash-1-3-128` |
/// | siphash | siphash 2-4 | 64   | `siphash-2-4-64`  |
/// | siphash | siphash 2-4 | 128  | `siphash-2-4-128` |
/// | xxh     | xxh32       | 32   | `xxh-32`          |
/// | xxh     | xxh64       | 64   | `xxh-64`          |
/// | xxh     | xxh3        | 64   | `xxh3-64`         |
/// | xxh     | xxh3        | 128  | `xxh3-128`        |
/// | md      | md5         | 128  | `md5`             |
/// | sha     | sha1        | 160  | `sha1`            |
/// | sha     | sha2        | 224  | `sha2-224`        |
/// | sha     | sha2        | 256  | `sha2-256`        |
/// | sha     | sha2        | 384  | `sha2-384`        |
/// | sha     | sha2        | 512  | `sha2-512`        |
/// | sha     | sha2        | 224  | `sha2-512/224`    |
/// | sha     | sha2        | 256  | `sha2-512/256`    |
/// | sha     | shake       | 128  | `shake-128`       |
/// | sha     | shake       | 256  | `shake-256`       |
/// | sha     | turboshake  | 128  | `turboshake-128`  |
/// | sha     | turboshake  | 256  | `turboshake-256`  |
/// | sha     | cshake      | 128  | `cshake-128`      |
/// | sha     | cshake      | 256  | `cshake-256`      |
/// | sha     | keccak      | 224  | `keccak-224`      |
/// | sha     | keccak      | 256  | `keccak-256`      |
/// | sha     | keccak      | 256  | `keccak-256-full` |
/// | sha     | keccak      | 384  | `keccak-384`      |
/// | sha     | keccak      | 512  | `keccak-512`      |
/// | sha     | sha3        | 224  | `sha3-224`        |
/// | sha     | sha3        | 256  | `sha3-256`        |
/// | sha     | sha3        | 384  | `sha3-384`        |
/// | sha     | sha3        | 512  | `sha3-512`        |
/// | blake   | blake2s     | 256  | `blake2s`         |
/// | blake   | blake2b     | 512  | `blake2b`         |
/// | blake   | blake3      | 256  | `blake3`          |
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    #[cfg(feature = "hashes-seahash")]
    Seahash,

    #[cfg(feature = "hashes-crc32")]
    Crc32,

    #[cfg(feature = "hashes-crc32c")]
    Crc32c,

    #[cfg(feature = "hashes-siphash")]
    Siphash_1_3_64,

    #[cfg(feature = "hashes-siphash")]
    Siphash_1_3_128,

    #[cfg(feature = "hashes-siphash")]
    Siphash_2_4_64,

    #[cfg(feature = "hashes-siphash")]
    Siphash_2_4_128,

    #[cfg(feature = "hashes-xxh")]
    Xxh_32,

    #[cfg(feature = "hashes-xxh")]
    Xxh_64,

    #[cfg(feature = "hashes-xxh")]
    Xxh3_64,

    #[cfg(feature = "hashes-xxh")]
    Xxh3_128,

    #[cfg(feature = "hashes-md5")]
    Md5,

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
    TurboShake_128,

    #[cfg(feature = "hashes-sha3")]
    TurboShake_256,

    #[cfg(feature = "hashes-sha3")]
    CShake_128,

    #[cfg(feature = "hashes-sha3")]
    CShake_256,

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

    #[cfg(feature = "hashes-blake2")]
    Blake2s,

    #[cfg(feature = "hashes-blake2")]
    Blake2b,

    #[cfg(feature = "hashes-blake3")]
    Blake3
}

impl HashAlgorithm {
    pub const fn name(&self) -> &'static str {
        match self {
            #[cfg(feature = "hashes-seahash")]
            Self::Seahash => "seahash",

            #[cfg(feature = "hashes-crc32")]
            Self::Crc32 => "crc32",

            #[cfg(feature = "hashes-crc32c")]
            Self::Crc32c => "crc32c",

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
            Self::TurboShake_128 => "turboshake-128",

            #[cfg(feature = "hashes-sha3")]
            Self::TurboShake_256 => "turboshake-256",

            #[cfg(feature = "hashes-sha3")]
            Self::CShake_128 => "cshake-128",

            #[cfg(feature = "hashes-sha3")]
            Self::CShake_256 => "cshake-256",

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
            Self::Blake2s => "blake2s",

            #[cfg(feature = "hashes-blake2")]
            Self::Blake2b => "blake2b",

            #[cfg(feature = "hashes-blake2")]
            Self::Blake3 => "blake3"
        }
    }
}

impl FromStr for HashAlgorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            #[cfg(feature = "hashes-seahash")]
            "seahash" => Ok(Self::Seahash),

            #[cfg(feature = "hashes-crc32")]
            "crc32" => Ok(Self::Crc32),

            #[cfg(feature = "hashes-crc32c")]
            "crc32c" => Ok(Self::Crc32c),

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
            "turboshake-128" | "turboshake128" => Ok(Self::TurboShake_128),

            #[cfg(feature = "hashes-sha3")]
            "turboshake-256" | "turboshake256" => Ok(Self::TurboShake_256),

            #[cfg(feature = "hashes-sha3")]
            "cshake-128" | "cshake128" => Ok(Self::CShake_128),

            #[cfg(feature = "hashes-sha3")]
            "cshake-256" | "cshake256" => Ok(Self::CShake_256),

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
            "blake2s" => Ok(Self::Blake2s),

            #[cfg(feature = "hashes-blake2")]
            "blake2b" => Ok(Self::Blake2b),

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
