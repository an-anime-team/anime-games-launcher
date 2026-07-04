// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-packages
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

use std::collections::VecDeque;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use base32::Alphabet;

const ALPHABET: Alphabet = Alphabet::Rfc4648HexLower {
    padding: false
};

/// Standard `agl-packages` hash format type.
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash([u8; Self::SIZE]);

impl Hash {
    /// Hash size in bytes.
    pub const SIZE: usize = 16;

    /// Get random hash.
    pub fn rand() -> Self {
        let mut hash = [0; 16];

        fastrand::fill(&mut hash);

        Self(hash)
    }

    /// Calculate hash value from the given bytes buffer.
    #[inline]
    pub fn digitize(buf: &[u8]) -> Self {
        let mut hash = [0; Self::SIZE];

        hash.copy_from_slice(&blake3::hash(buf).as_slice()[..Self::SIZE]);

        Self(hash)
    }

    /// Calculate hash value from provided filesystem entry.
    pub fn digitize_path(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        fn update_hasher(
            hasher: &mut Hasher,
            file: &mut File
        ) -> std::io::Result<()> {
            let mut buf = [0; 4096];

            loop {
                let n = file.read(&mut buf)?;

                if n == 0 {
                    break;
                }

                hasher.update(&buf[..n]);
            }

            Ok(())
        }

        let mut hasher = Hasher::default();

        let mut queue = VecDeque::<(PathBuf, bool)>::new();

        // Do not include filename of the entry path since it can be e.g.
        // a randomly generated archive folder name, or a name of the file
        // which we want to hash. All the nested files and folders, on the other
        // hand, should match both filename and content.
        queue.push_back((path.into(), false));

        while let Some((path, include_name)) = queue.pop_front() {
            let path = path.canonicalize()?;

            // Write the filename to the hasher.
            if include_name
                && let Some(filename) = path.file_name().and_then(|name| name.to_str())
            {
                hasher.update(filename.as_bytes());
            }

            // Update the hasher from the file's content.
            if path.is_file() {
                let mut file = File::open(&path)?;

                update_hasher(&mut hasher, &mut file)?;
            }

            // Iterate over the folder's entries.
            else {
                // Resolve all the first level folder entries.
                let mut paths = path.read_dir()?
                    .map(|entry| {
                        entry.map(|entry| (entry.path(), true))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Make sure that all the paths will be appended in the same
                // order.
                paths.sort();

                // Push them to the end of the queue.
                queue.extend(paths);
            }
        }

        Ok(hasher.finish())
    }

    /// Encode current hash into a nix-base32 string.
    #[inline]
    pub fn to_base32(&self) -> String {
        nix_base32::to_nix_base32(&self.0)
    }

    /// Try to decode a hash from the given nix-base32 string.
    pub fn from_base32(str: impl AsRef<str>) -> Option<Self> {
        nix_base32::from_nix_base32(str.as_ref())
            .or_else(|| base32::decode(ALPHABET, str.as_ref()))
            .map(|buf| {
                let mut hash = [0; Self::SIZE];

                if buf.len() >= Self::SIZE {
                    hash.copy_from_slice(&buf[..Self::SIZE]);
                } else {
                    hash[..buf.len()].copy_from_slice(&buf);
                }

                Self(hash)
            })
    }
}

impl std::str::FromStr for Hash {
    type Err = ();

    #[inline]
    fn from_str(hash: &str) -> Result<Self, Self::Err> {
        Hash::from_base32(hash).ok_or(())
    }
}

impl std::fmt::Debug for Hash {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.to_base32())
    }
}

impl std::fmt::Display for Hash {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_base32())
    }
}

impl std::ops::Deref for Hash {
    type Target = [u8; Self::SIZE];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::BitXor for Hash {
    type Output = Self;

    #[inline]
    fn bitxor(mut self, rhs: Self) -> Self::Output {
        self.0.iter_mut()
            .zip(rhs.0.iter())
            .for_each(|(a, b)| *a ^= b);

        self
    }
}

impl std::ops::BitXorAssign for Hash {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0.iter_mut()
            .zip(rhs.0.iter())
            .for_each(|(a, b)| *a ^= b);
    }
}

/// Standard `agl-packages` hasher.
#[derive(Default)]
pub struct Hasher(blake3::Hasher);

impl Hasher {
    #[inline]
    pub fn update(&mut self, buf: &[u8]) {
        self.0.update(buf);
    }

    #[inline]
    pub fn finish(&self) -> Hash {
        let mut hash = [0; Hash::SIZE];

        hash.copy_from_slice(&self.0.finalize().as_slice()[..Hash::SIZE]);

        Hash(hash)
    }
}

#[test]
fn test() {
    let hash = Hash::digitize(b"Hello, World!");

    assert_eq!(hash.to_base32(), "vk3d0ph9av12s");

    assert_eq!(Hash::from_base32(hash.to_base32()), Some(hash));
}

// TODO: test Hash::from_path
