use serde::{Serialize, Deserialize};

use std::path::{Path, PathBuf};
use std::hash::Hasher;
use std::io::Read;

/// Provide custom object hashing mechanism.
pub trait AsHash {
    /// Calculate unique hash of the object.
    fn hash(&self) -> Hash;

    /// Calculate partial hash of the object.
    ///
    /// Partial hashes verify only most important
    /// parts of the data. They ignore things like
    /// metadata, creation timestamps and so on.
    /// Actual value depends on implementation.
    fn partial_hash(&self) -> Hash {
        self.hash()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
/// Wrapper around an integer used as hash value.
pub struct Hash(pub u64);

impl Hash {
    #[inline]
    /// Generate new random hash.
    pub fn rand() -> Self {
        Self(rand::random())
    }

    #[inline]
    /// Chain two hashes together, making a new one.
    pub fn chain(self, other: Hash) -> Self {
        self ^ other
    }

    #[inline]
    /// Generate hash from the given data buffer.
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

        // Otherwise expect it to be a folder and handle it
        // by hashing each individual file and names of files
        // and folders, and xoring all the values together.
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
    pub fn to_base32(&self) -> String {
        base32::encode(base32::Alphabet::Rfc4648HexLower { padding: false }, &self.0.to_be_bytes())
    }

    /// Try to parse hash value from the base32 string.
    pub fn from_base32(str: impl AsRef<str>) -> Option<Self> {
        let mut buf = [0; 8];

        let hash = base32::decode(base32::Alphabet::Rfc4648HexLower { padding: false }, str.as_ref())?;

        buf.copy_from_slice(&hash[..8]);

        Some(Self(u64::from_be_bytes(buf)))
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

macro_rules! impl_as_hash {
    (num $($type:ty)*) => {
        $(
            impl AsHash for $type {
                fn hash(&self) -> Hash {
                    Hash::for_slice(self.to_be_bytes())
                }
            }
        )*
    };

    (bytes $($type:ty)*) => {
        $(
            impl AsHash for $type {
                fn hash(&self) -> Hash {
                    Hash::for_slice(self.as_bytes())
                }
            }
        )*
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
        Hash::for_slice(&self)
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
        T::hash(&self)
    }

    #[inline]
    fn partial_hash(&self) -> Hash {
        T::partial_hash(&self)
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
    use crate::core::prelude::*;

    use super::*;

    #[tokio::test]
    async fn hash() -> Result<(), DownloaderError> {
        let path = std::env::temp_dir().join(".agl-hash-test");
        let folder = std::env::temp_dir().join(".agl-hash-test-folder");

        if !path.exists() {
            Downloader::new("https://github.com/doitsujin/dxvk/releases/download/v2.4/dxvk-2.4.tar.gz")?
                .with_output_file(&path)
                .download(|_, _, _| {})
                .await?
                .wait()?;
        }

        if !folder.exists() {
            TarArchive::open(&path)?
                .extract(&folder, |_, _, _| {})?
                .wait().unwrap();
        }

        assert_eq!(Hash::for_entry(&path)?, Hash(13290421503141924848));
        assert_eq!(Hash::for_entry(&folder)?, Hash(17827013605004440863));

        Ok(())
    }

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
