use std::path::{Path, PathBuf};
use std::hash::Hasher;
use std::io::Read;

use wineyard_core::export::hashes::seahash;

use serde::{Serialize, Deserialize};
use base32::Alphabet;

const ALPHABET: Alphabet = Alphabet::Rfc4648HexLower {
    padding: false
};

/// Provide custom object hashing mechanism.
pub trait AsHash {
    /// Calculate unique hash of the object.
    fn hash(&self) -> Hash;

    /// Calculate partial hash of the object.
    ///
    /// Partial hashes verify only most important parts of the data. They ignore
    /// things like metadata, creation timestamps and so on. Actual value
    /// depends on implementation.
    #[inline(always)]
    fn partial_hash(&self) -> Hash {
        self.hash()
    }
}

/// Wrapper around an integer used as hash value.
#[derive(
    Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
    Serialize, Deserialize
)]
pub struct Hash(pub u64);

impl Hash {
    /// Generate new random hash.
    #[inline]
    pub fn rand() -> Self {
        Self(rand::random())
    }

    /// Chain two hashes together, making a new one.
    #[inline]
    pub fn chain(self, other: impl Into<Hash>) -> Self {
        self ^ other.into()
    }

    /// Generate hash from the given data buffer.
    #[inline]
    pub fn for_slice(buf: impl AsRef<[u8]>) -> Self {
        let hash = seahash::hash(buf.as_ref());

        Self(hash)
    }

    /// Generate hash for a given filesystem entry.
    pub fn for_entry(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let mut path: PathBuf = path.into();

        // Resolve symlinks before hashing.
        while path.is_symlink() {
            path = path.read_link()?;
        }

        fn hash_file(path: &Path) -> std::io::Result<Hash> {
            let mut file = std::fs::File::open(path)?;
            let mut hasher = seahash::SeaHasher::new();

            let mut buf = [0; 4096];

            loop {
                let len = file.read(&mut buf)?;

                if len == 0 {
                    break;
                }

                hasher.write(&buf[..len]);
            }

            Ok(Hash(hasher.finish()))
        }

        // Handle file by hashing it by chunks.
        if path.is_file() {
            return hash_file(&path);
        }

        // Otherwise expect it to be a folder and handle it by hashing each
        // individual file and names of files and folders, and xoring all the
        // values together.
        let root = path.clone();

        let mut folders = vec![path];
        let mut result = 0;

        while let Some(path) = folders.pop() {
            for entry in path.read_dir()?.flatten() {
                let mut path = entry.path();

                // Resolve symlinks before hashing.
                while path.is_symlink() {
                    path = path.read_link()?;
                }

                // Hash the file and xor it with the result value.
                if path.is_file() {
                    // Hash the file's relative path to ensure the structure.
                    if let Ok(name) = path.strip_prefix(&root) {
                        result ^= seahash::hash(name.as_os_str().as_encoded_bytes());
                    }

                    // Hash the file's content.
                    result ^= hash_file(&path)?.0;
                }

                // Otherwise it's a folder and we need to push it
                // to the hasing queue.
                else {
                    // Hash the folder's relative path to ensure the structure.
                    if let Ok(name) = path.strip_prefix(&root) {
                        result ^= seahash::hash(name.as_os_str().as_encoded_bytes());
                    }

                    folders.push(path);
                }
            }
        }

        Ok(Self(result))
    }

    /// Convert hash to the base32 string.
    #[inline]
    pub fn to_base32(&self) -> String {
        base32::encode(ALPHABET, &self.0.to_be_bytes())
    }

    /// Try to parse hash value from the base32 string.
    pub fn from_base32(str: impl AsRef<str>) -> Option<Self> {
        let mut buf = [0; 8];

        let hash = base32::decode(ALPHABET, str.as_ref())?;

        buf.copy_from_slice(&hash[..8]);

        Some(Self(u64::from_be_bytes(buf)))
    }
}

impl std::fmt::Display for Hash {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_base32())
    }
}

impl AsRef<Hash> for Hash {
    #[inline(always)]
    fn as_ref(&self) -> &Hash {
        self
    }
}

impl AsRef<u64> for Hash {
    #[inline(always)]
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl std::ops::BitXor for Hash {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Hash(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for Hash {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

macro_rules! impl_from_num {
    ($($type:ty)+) => {
        $(
            impl From<$type> for Hash {
                #[inline(always)]
                fn from(value: $type) -> Self {
                    Self(value as u64)
                }
            }
        )+
    };
}

impl_from_num!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128);

macro_rules! impl_as_hash {
    (num $($type:ty)+) => {
        $(
            impl AsHash for $type {
                fn hash(&self) -> Hash {
                    Hash::for_slice(self.to_be_bytes())
                }
            }
        )+
    };

    (bytes $($type:ty)+) => {
        $(
            impl AsHash for $type {
                fn hash(&self) -> Hash {
                    Hash::for_slice(self.as_bytes())
                }
            }
        )+
    };
}

impl_as_hash!(num u16 u32 u64 u128 i8 i16 i32 i64 i128);
impl_as_hash!(bytes &str String);

impl AsHash for Hash {
    #[inline]
    fn hash(&self) -> Hash {
        *self
    }
}

impl AsHash for [u8] {
    #[inline]
    fn hash(&self) -> Hash {
        Hash::for_slice(self)
    }
}

impl AsHash for &[u8] {
    #[inline]
    fn hash(&self) -> Hash {
        Hash::for_slice(self)
    }
}

impl AsHash for Vec<u8> {
    #[inline]
    fn hash(&self) -> Hash {
        Hash::for_slice(self)
    }
}

impl AsHash for Box<[u8]> {
    #[inline]
    fn hash(&self) -> Hash {
        Hash::for_slice(self)
    }
}

impl<T: AsHash> AsHash for Box<T> {
    #[inline]
    fn hash(&self) -> Hash {
        T::hash(self)
    }

    #[inline]
    fn partial_hash(&self) -> Hash {
        T::partial_hash(self)
    }
}

impl<T: AsHash> AsHash for Option<T> {
    #[inline]
    fn hash(&self) -> Hash {
        self.as_ref()
            .map(AsHash::hash)
            .unwrap_or_default()
    }

    #[inline]
    fn partial_hash(&self) -> Hash {
        self.as_ref()
            .map(AsHash::partial_hash)
            .unwrap_or_default()
    }
}

impl<T: AsHash> AsHash for [T] {
    #[inline]
    fn hash(&self) -> Hash {
        self.iter()
            .map(T::hash)
            .fold(Hash::default(), Hash::chain)
    }

    #[inline]
    fn partial_hash(&self) -> Hash {
        self.iter()
            .map(T::partial_hash)
            .fold(Hash::default(), Hash::chain)
    }
}

impl<T: AsHash> AsHash for &[T] {
    #[inline]
    fn hash(&self) -> Hash {
        self.iter()
            .map(T::hash)
            .fold(Hash::default(), Hash::chain)
    }

    #[inline]
    fn partial_hash(&self) -> Hash {
        self.iter()
            .map(T::partial_hash)
            .fold(Hash::default(), Hash::chain)
    }
}

impl<T: AsHash> AsHash for Vec<T> {
    #[inline]
    fn hash(&self) -> Hash {
        self.iter()
            .map(T::hash)
            .fold(Hash::default(), Hash::chain)
    }

    #[inline]
    fn partial_hash(&self) -> Hash {
        self.iter()
            .map(T::partial_hash)
            .fold(Hash::default(), Hash::chain)
    }
}

impl<T: AsHash> AsHash for std::collections::HashSet<T> {
    #[inline]
    fn hash(&self) -> Hash {
        self.iter()
            .map(T::hash)
            .fold(Hash::default(), Hash::chain)
    }

    #[inline]
    fn partial_hash(&self) -> Hash {
        self.iter()
            .map(T::partial_hash)
            .fold(Hash::default(), Hash::chain)
    }
}

impl<K: AsHash, V: AsHash> AsHash for std::collections::HashMap<K, V> {
    #[inline]
    fn hash(&self) -> Hash {
        self.iter()
            .map(|(k, v)| k.hash().chain(v.hash()))
            .fold(Hash::default(), Hash::chain)
    }

    #[inline]
    fn partial_hash(&self) -> Hash {
        self.iter()
            .map(|(k, v)| k.partial_hash().chain(v.partial_hash()))
            .fold(Hash::default(), Hash::chain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_hash() {
        assert_eq!(123456789_u64.hash(), Hash(16531472742531055538));
        assert_eq!("Hello, World!".hash(), Hash(3369351306556737277));
        assert_eq!(Some(123456_u32).hash(), Hash(13440295563441507171));
        assert_eq!(None::<String>.hash(), Hash(0));
        assert_eq!([1_i16, -2, 3].hash(), Hash(7455816351535058648));
    }

    #[test]
    fn base32() {
        assert_eq!(Hash::for_slice(b"Hello, World!").to_base32(), "5r15eab6q03fq");
        assert_eq!(Hash::from_base32("5r15eab6q03fq"), Some(Hash(3369351306556737277)));
        assert_eq!(Hash::from_base32("Hello, World!"), None);
    }
}
