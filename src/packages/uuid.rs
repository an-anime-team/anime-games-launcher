use super::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Uuid(uuid::Uuid);

impl Uuid {
    #[inline]
    /// Build UUID using given string as a seed
    /// 
    /// Same seed will always generate the same value
    /// 
    /// Use `Uuid::try_from` if you want to parse an existing UUID
    pub fn new_from_str(str: impl AsRef<str>) -> Self {
        Self::new_from_slice(str.as_ref().as_bytes())
    }

    #[inline]
    /// Build UUID using given hash as a seed
    /// 
    /// Same seed will always generate the same value
    pub fn new_from_hash(hash: &Hash) -> Self {
        Self::new_from_slice(hash.bytes())
    }

    /// Build UUID using given bytes slice as a seed
    /// 
    /// Same seed will always generate the same value
    pub fn new_from_slice(slice: &[u8]) -> Self {
        let mut bytes = [0; 16];

        for (i, byte) in slice.iter().enumerate() {
            bytes[i % 16] ^= *byte;
        }

        Self(uuid::Uuid::from_bytes(bytes))
    }
}

impl TryFrom<&str> for Uuid {
    type Error = uuid::Error;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(uuid::Uuid::parse_str(value)?))
    }
}

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
