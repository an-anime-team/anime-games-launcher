#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HashAlgorithm {
    Xxh3,
    // Sha1,
    // Md5,
    Crc32
}

impl Default for HashAlgorithm {
    #[inline]
    fn default() -> Self {
        Self::Xxh3
    }
}

impl HashAlgorithm {
    #[inline]
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Xxh3  => "xxh3",
            // Self::Sha1  => "sha1",
            // Self::Md5   => "md5",
            Self::Crc32 => "crc32"
        }
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: impl AsRef<str>) -> Option<Self> {
        match str.as_ref() {
            "xxh3"  => Some(Self::Xxh3),
            // "sha1"  => Some(Self::Sha1),
            // "md5"   => Some(Self::Md5),
            "crc32" => Some(Self::Crc32),

            _ => None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hash {
    algorithm: HashAlgorithm,
    value: u64
}

impl Hash {
    pub fn from_slice(algorithm: HashAlgorithm, slice: impl AsRef<[u8]>) -> Self {
        Self {
            algorithm,
            value: match algorithm {
                HashAlgorithm::Xxh3 => xxhash_rust::xxh3::xxh3_64(slice.as_ref()),

                HashAlgorithm::Crc32 => {
                    let hash = crc32fast::hash(slice.as_ref()) as u64;

                    (hash << 32) | (!hash)
                }
            }
        }
    }

    #[inline]
    pub fn algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }

    #[inline]
    pub fn value(&self) -> u64 {
        self.value
    }
}

impl TryFrom<&str> for Hash {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (algorithm, value) = value
            .split_once('-')
            .unwrap_or((HashAlgorithm::Xxh3.to_str(), value));

        let mut bytes = [0; 8];

        let hash = base32::decode(base32::Alphabet::Rfc4648Lower { padding: false }, value)
            .ok_or_else(|| anyhow::anyhow!("Failed to decode hash value from base32"))?;

        if hash.len() != 8 {
            anyhow::bail!("Hash value has incorrect bytes length. Expected 8 bytes, got {}", hash.len());
        }

        bytes.copy_from_slice(&hash[..8]);

        Ok(Self {
            algorithm: HashAlgorithm::from_str(algorithm)
                .ok_or_else(|| anyhow::anyhow!("Unsupported hash algorithm: {algorithm}"))?,

            value: u64::from_be_bytes(bytes)
        })
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.algorithm.to_str(), base32::encode(base32::Alphabet::Rfc4648Lower { padding: false }, &self.value.to_be_bytes()))
    }
}
