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

use std::path::{Path, PathBuf};

use crate::hash::Hash;

/// Anime Games Launcher packages storage.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Storage {
    path: PathBuf
}

impl Storage {
    /// Try to open a packages storage within provided folder path. Create the
    /// folder if it doesn't exist.
    pub fn open(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let path: PathBuf = path.into();

        if !path.is_dir() {
            std::fs::create_dir_all(&path)?;
        }

        Ok(Self {
            path
        })
    }

    /// Store a resource from provided filesystem entry path. This can be either
    /// a file or a folder, symlinks are resolved. Original entry is kept
    /// untouched.
    ///
    /// Return hash of the stored resource.
    pub fn store(&self, path: impl Into<PathBuf>) -> std::io::Result<Hash> {
        fn try_copy(source: &Path, target: &Path) -> std::io::Result<()> {
            if source.is_file() {
                std::fs::copy(source, target)?;
            }

            else if source.is_dir() {
                std::fs::create_dir_all(target)?;

                for entry in source.read_dir()? {
                    let entry = entry?;

                    try_copy(&entry.path(), &target.join(entry.file_name()))?;
                }
            }

            else if source.is_symlink() {
                // FIXME: only works on unix systems while we target to support
                //        all the OSes.

                #[allow(clippy::collapsible_if)]
                if let Some(source_filename) = source.file_name() {
                    std::os::unix::fs::symlink(
                        source.read_link()?,
                        target.join(source_filename)
                    )?;
                }
            }

            Ok(())
        }

        let path: PathBuf = path.into();

        // Calculate hash of the resource file / folder.
        let hash = Hash::from_path(path.clone())?;

        // Copy the resource into the storage folder.
        try_copy(&path, &self.path.join(hash.to_base32()))?;

        Ok(hash)
    }

    /// Check if storage has a content for a resource with provided hash.
    ///
    /// Note that this method *does not* verify hash value of the stored
    /// content.
    #[inline]
    pub fn has_resource(&self, hash: &Hash) -> bool {
        self.path.join(hash.to_base32()).exists()
    }

    /// Verify that resource content for provided hash is valid.
    ///
    /// If resource with provided hash is not stored, then `Ok(false)` is
    /// returned.
    pub fn verify_resource(&self, hash: &Hash) -> std::io::Result<bool> {
        let path = self.path.join(hash.to_base32());

        if !path.exists() {
            return Ok(false);
        }

        Ok(&Hash::from_path(path)? == hash)
    }
}
