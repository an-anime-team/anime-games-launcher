// SPDX-License-Identifier: GPL-3.0-or-later
//
// agl-packages
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

use std::hash::Hasher as _;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use base32::Alphabet;

const ALPHABET: Alphabet = Alphabet::Rfc4648HexLower {
    padding: false
};

/// Standard `agl-packages` hash format type.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash(u64);

impl Hash {
    /// Get random hash.
    #[inline]
    pub fn rand() -> Self {
        Self(fastrand::u64(..))
    }

    /// Calculate hash value from the given bytes buffer.
    pub fn from_bytes(buf: &[u8]) -> Self {
        Self(seahash::hash(buf))
    }

    /// Calculate hash value from provided filesystem entry.
    pub fn from_path(path: impl Into<PathBuf>) -> std::io::Result<Self> {
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

    /// Encode current hash into a base32 string.
    #[inline]
    pub fn to_base32(&self) -> String {
        base32::encode(ALPHABET, &self.0.to_le_bytes())
    }

    /// Try to decode a hash from the given base32 string.
    pub fn from_base32(str: impl AsRef<str>) -> Option<Self> {
        base32::decode(ALPHABET, str.as_ref())
            .and_then(|buf| {
                if buf.len() != 8 {
                    return None;
                }

                let mut hash = [0; 8];

                hash.copy_from_slice(&buf);

                Some(Self(u64::from_le_bytes(hash)))
            })
    }
}

impl std::fmt::Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_base32())
    }
}

impl std::ops::Deref for Hash {
    type Target = u64;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Standard `agl-packages` hasher.
#[derive(Default)]
pub struct Hasher(seahash::SeaHasher);

impl Hasher {
    #[inline]
    pub fn update(&mut self, buf: &[u8]) {
        self.0.write(buf);
    }

    #[inline]
    pub fn finish(&self) -> Hash {
        Hash(self.0.finish())
    }
}

#[test]
fn test() {
    let hash = Hash::from_bytes(b"Hello, World!");

    assert_eq!(hash.to_base32(), "vk3d0ph9av12s");

    assert_eq!(Hash::from_base32(hash.to_base32()), Some(hash));
}

// TODO: test Hash::from_path
