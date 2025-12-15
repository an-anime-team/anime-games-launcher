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

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde_json::Value as Json;

use agl_core::network::downloader::{
    Downloader, DownloadOptions, DownloaderError
};

use agl_core::archives::{Archive, ArchiveFormat, ArchiveError};

use crate::hash::Hash;
use crate::format::ResourceFormat;
use crate::package::PackageManifest;
use crate::lock::{Lock, LockedPackageInfo, LockedResourceInfo};

#[derive(Debug, thiserror::Error)]
pub enum InstallPackagesError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("downloader error: {0}")]
    Downloader(#[from] DownloaderError),

    #[error("failed to deserialize package manifest: {0}")]
    Deserialize(#[from] serde_json::Error),

    #[error(
        "failed to decode package manifest from a json object; url = '{url}', hash = '{}'",
        hash.to_base32()
    )]
    DecodeManifest {
        url: String,
        hash: Hash
    },

    #[error(
        "resource '{url}' has hash '{}' while '{}' was expected",
        actual.to_base32(),
        expected.to_base32()
    )]
    ResourceHashMismatch {
        actual: Hash,
        expected: Hash,
        url: String
    },

    #[error("unsupported resource archive format: '{url}'")]
    ArchiveFormatUnsupported {
        url: String
    },

    #[error("failed to extract resource archive: {0}")]
    ArchiveExtract(#[from] ArchiveError),

    #[error(
        "package '{package_url}' with hash '{}' tried to reference a resource with name '{resource_name}', but this name is already used by another resource",
        package_hash.to_base32()
    )]
    PackageNameAlreadyUsed {
        package_url: String,
        package_hash: Hash,
        resource_name: String
    }
}

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

    /// Get storage folder path.
    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get path to a resource with provided hash.
    #[inline]
    pub fn resource_path(&self, hash: &Hash) -> PathBuf {
        self.path.join(hash.to_base32())
    }

    /// Store a resource from provided filesystem entry path. This can be either
    /// a file or a folder, symlinks are resolved. Original entry is kept
    /// untouched.
    ///
    /// Return hash of the stored resource.
    pub fn store_resource(&self, path: impl Into<PathBuf>) -> std::io::Result<Hash> {
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
        try_copy(&path, &self.resource_path(&hash))?;

        Ok(hash)
    }

    /// Check if storage has a content for a resource with provided hash.
    ///
    /// Note that this method *does not* verify hash value of the stored
    /// content.
    #[inline]
    pub fn has_resource(&self, hash: &Hash) -> bool {
        self.resource_path(hash).exists()
    }

    /// Verify that resource content for provided hash is valid.
    ///
    /// If resource with provided hash is not stored, then `Ok(false)` is
    /// returned.
    pub fn verify_resource(&self, hash: &Hash) -> std::io::Result<bool> {
        let path = self.resource_path(hash);

        if !path.exists() {
            return Ok(false);
        }

        Ok(&Hash::from_path(path)? == hash)
    }

    /// Verify that the storage has all the resources listed in the provided
    /// lock and that their hashes are valid.
    pub fn verify_lock(&self, lock: &Lock) -> std::io::Result<bool> {
        for hash in lock.resources.keys() {
            if !self.verify_resource(hash)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Install packages to the current storage and provide a lock for them.
    pub async fn install_packages<T: ToString>(
        &self,
        downloader: &Downloader,
        urls: impl IntoIterator<Item = T>
    ) -> Result<Lock, InstallPackagesError> {
        /// Normalize given URL.
        #[inline]
        fn normalize_url(url: impl AsRef<str>) -> String {
            let (scheme, url) = url.as_ref()
                .split_once("://")
                .map(|(scheme, url)| (Some(scheme), url))
                .unwrap_or((None, url.as_ref()));

            let url = url
                .replace('\\', "/")
                .replace("/./", "/")
                .replace("//", "/");

            let url = url.split('/')
                .collect::<Vec<_>>();

            let mut clean_parts = Vec::with_capacity(url.len());

            let mut i = 0;
            let n = url.len() - 1;

            while i < n {
                if url[i + 1] == ".." {
                    i += 2;

                    continue;
                }

                clean_parts.push(url[i]);

                i += 1;
            }

            clean_parts.push(url[n]);

            let url = clean_parts.join("/");

            if let Some(scheme) = scheme {
                format!("{scheme}://{url}")
            } else {
                url
            }
        }

        // Create [url] => [hash] table.
        let mut resource_hashes = HashMap::new();

        // Create packages and resources downloading queues.
        //
        // We will loop through the packages, order their processing, then
        // iterate over the packages' manifests and order their resources'
        // processing, at each step modifying the packaages and resources
        // queues.
        //
        // If some package uses another package as a dependency - then packages
        // queue will be updated and the first loop will repeat until it's
        // empty.
        let mut packages_queue = Vec::new();
        let mut resources_queue = Vec::new();

        // Create packages and resources lock info tables.
        let mut packages_lock_info = HashMap::new();
        let mut resources_lock_info = HashMap::new();

        // Create table of processed resources.
        let mut processed_resources = HashSet::new();

        // Prepare root packages list.
        let root_packages = urls.into_iter()
            .map(|url| normalize_url(url.to_string()))
            .collect::<HashSet<String>>();

        // Push root packages to the processing queue.
        packages_queue.extend(root_packages.clone());

        // Loop while there are packages to process.
        while !packages_queue.is_empty() {
            // Iterate over the packages URLs.
            let mut tasks = Vec::new();

            for package_url in packages_queue.drain(..) {
                // Skip already downloaded packages.
                if processed_resources.contains(&(package_url.clone(), ResourceFormat::Package)) {
                    continue;
                }

                // Prepare a temp path for the package's manifest file.
                let temp_path = self.resource_path(&Hash::rand());

                // Start downloading the package's manifest.
                let task = downloader.download_with_options(
                    &package_url,
                    &temp_path,
                    DownloadOptions {
                        continue_download: false,
                        on_update: None,
                        on_finish: None
                    }
                );

                tasks.push((
                    task,
                    package_url,
                    temp_path
                ));
            }

            // Await the manifests downloading tasks.
            for (task, package_url, temp_path) in tasks.drain(..) {
                // Wait until downloading is done.
                task.wait().await?;

                // Read the manifest file and calculate its hash.
                let manifest = std::fs::read(&temp_path)?;
                let manifest_hash = Hash::from_bytes(&manifest);

                // Deserialize package manifest.
                let manifest = serde_json::from_slice::<Json>(&manifest)?;

                let manifest = PackageManifest::from_json(&manifest)
                    .ok_or_else(|| {
                        InstallPackagesError::DecodeManifest {
                            url: package_url.clone(),
                            hash: manifest_hash
                        }
                    })?;

                // Store the manifest in the storage.
                self.store_resource(&temp_path)?;

                // Delete temporary file.
                let _ = std::fs::remove_file(&temp_path);

                // List this package as processed.
                processed_resources.insert((package_url.clone(), ResourceFormat::Package));

                // Link requested URL with its output hash.
                resource_hashes.insert(package_url.clone(), manifest_hash);

                // Obtain the parent "folder" from the package manifest URL.
                let (parent_url, _) = package_url.rsplit_once('/')
                    .unwrap_or((package_url.as_str(), ""));

                // Update the package's info in the lock table.
                let (_, mut inputs, mut outputs) = packages_lock_info.remove(&manifest_hash)
                    .unwrap_or_else(|| {
                        (String::new(), HashMap::new(), HashMap::new())
                    });

                // Update package inputs.
                for (name, resource_info) in manifest.inputs.iter() {
                    if inputs.contains_key(name) {
                        return Err(InstallPackagesError::PackageNameAlreadyUsed {
                            package_url,
                            package_hash: manifest_hash,
                            resource_name: name.clone()
                        });
                    }

                    // Prepare resource URL.
                    let mut resource_url = if resource_info.uri.starts_with("http") {
                        resource_info.uri.clone()
                    } else {
                        format!("{parent_url}/{}", resource_info.uri)
                    };

                    // Normalize it.
                    resource_url = normalize_url(resource_url);

                    // Obtain the resource format.
                    let resource_format = resource_info.format
                        .unwrap_or_else(|| {
                            ResourceFormat::from_filename(&resource_url)
                        });

                    // Store the resource info.
                    inputs.insert(name.clone(), (
                        resource_url.clone(),
                        resource_format
                    ));

                    // And order its processing.
                    resources_queue.push((
                        resource_url.clone(),
                        resource_format,
                        resource_info.clone()
                    ));
                }

                // Update package outputs.
                for (name, resource_info) in manifest.outputs.iter() {
                    if outputs.contains_key(name) {
                        return Err(InstallPackagesError::PackageNameAlreadyUsed {
                            package_url,
                            package_hash: manifest_hash,
                            resource_name: name.clone()
                        });
                    }

                    // Prepare resource URL.
                    let mut resource_url = if resource_info.uri.starts_with("http") {
                        resource_info.uri.clone()
                    } else {
                        format!("{parent_url}/{}", resource_info.uri)
                    };

                    // Normalize it.
                    resource_url = normalize_url(resource_url);

                    // Obtain the resource format.
                    let resource_format = resource_info.format
                        .unwrap_or_else(|| {
                            ResourceFormat::from_filename(&resource_url)
                        });

                    // Store the resource URL.
                    outputs.insert(name.clone(), (
                        resource_url.clone(),
                        resource_format
                    ));

                    // And order its processing.
                    resources_queue.push((
                        resource_url.clone(),
                        resource_format,
                        resource_info.clone()
                    ));
                }

                // Insert updated info.
                packages_lock_info.insert(manifest_hash, (
                    package_url,
                    inputs,
                    outputs
                ));
            }

            // Iterate over the ordered resources.
            let mut tasks = Vec::new();

            for (resource_url, resource_format, resource_info) in resources_queue.drain(..) {
                // Skip already downloaded resources.
                if processed_resources.contains(&(resource_url.clone(), resource_format)) {
                    continue;
                }

                // If the resource is another package - order its processing
                // in the packages queue.
                if resource_format == ResourceFormat::Package {
                    // TODO: handle the expected resource_info.hash !!!!!
                    packages_queue.push(resource_url);

                    continue;
                }

                // Prepare a temp path for the resource.
                let temp_path = self.resource_path(&Hash::rand());

                // Start downloading the resource.
                let task = downloader.download_with_options(
                    &resource_url,
                    &temp_path,
                    DownloadOptions {
                        continue_download: false,
                        on_update: None,
                        on_finish: None
                    }
                );

                tasks.push((
                    task,
                    resource_url,
                    resource_format,
                    resource_info,
                    temp_path
                ));
            }

            // Await the resources downloading tasks.
            for (task, resource_url, resource_format, resource_info, temp_path) in tasks.drain(..) {
                // Wait until downloading is done.
                task.wait().await?;

                // Process the downloaded resource file according to its format.
                match resource_format {
                    ResourceFormat::File => {
                        // Calculate file hash.
                        let resource_hash = Hash::from_path(&temp_path)?;

                        // Compare expected file hash if it's provided with an
                        // actual hash and reject the file if it doesn't match.
                        if let Some(expected_hash) = resource_info.hash
                            && resource_hash != expected_hash
                        {
                            // Delete temporary file.
                            let _ = std::fs::remove_file(&temp_path);

                            return Err(InstallPackagesError::ResourceHashMismatch {
                                actual: resource_hash,
                                expected: expected_hash,
                                url: resource_url
                            });
                        }

                        // Store the downloaded file.
                        self.store_resource(&temp_path)?;

                        // Delete temporary file.
                        let _ = std::fs::remove_file(&temp_path);

                        // List this resource as processed.
                        processed_resources.insert((resource_url.clone(), resource_format));

                        // Link file URL with its hash.
                        resource_hashes.insert(resource_url.clone(), resource_hash);

                        // Reference this resource in the lock info table.
                        resources_lock_info.insert(resource_hash, resource_url);
                    }

                    ResourceFormat::Archive => {
                        // Prepare temporary archive extraction folder.
                        let temp_extract_path = self.resource_path(&Hash::rand());

                        std::fs::create_dir_all(&temp_extract_path)?;

                        // Try to predict the format of the archive from its
                        // download URL. It's needed because `temp_path` doesn't
                        // have any extension.
                        let Some(archive_format) = ArchiveFormat::from_filename(&resource_url) else {
                            return Err(InstallPackagesError::ArchiveFormatUnsupported {
                                url: resource_url
                            });
                        };

                        // Try to open the archive.
                        let Some(archive) = Archive::open_with_format(&temp_path, archive_format) else {
                            return Err(InstallPackagesError::ArchiveFormatUnsupported {
                                url: resource_url
                            });
                        };

                        // Extract archive to a temporary folder.
                        archive.extract(&temp_extract_path)?.wait()?;

                        // Delete temporary file.
                        let _ = std::fs::remove_file(&temp_path);

                        // Calculate hash of the extracted archive.
                        let resource_hash = Hash::from_path(&temp_extract_path)?;

                        // Compare expected file hash if it's provided with an
                        // actual hash and reject the file if it doesn't match.
                        if let Some(expected_hash) = resource_info.hash
                            && resource_hash != expected_hash
                        {
                            // Delete temporary extraction folder.
                            // (damn that's a scary function...)
                            let _ = std::fs::remove_dir_all(&temp_extract_path);

                            return Err(InstallPackagesError::ResourceHashMismatch {
                                actual: resource_hash,
                                expected: expected_hash,
                                url: resource_url
                            });
                        }

                        // Rename the temporary extraction folder into an actual
                        // one.
                        //
                        // TODO: make sane use of store_resource here somehow
                        std::fs::rename(
                            temp_extract_path,
                            self.resource_path(&resource_hash)
                        )?;

                        // List this resource as processed.
                        processed_resources.insert((resource_url.clone(), resource_format));

                        // Link file URL with its hash.
                        resource_hashes.insert(resource_url.clone(), resource_hash);

                        // Reference this resource in the lock info table.
                        resources_lock_info.insert(resource_hash, resource_url);
                    }

                    // We shouldn't get a package resource here.
                    ResourceFormat::Package => ()
                }
            }
        }

        Ok(Lock {
            root: root_packages.iter()
                // kinda guaranteed to always be Some(..)
                .flat_map(|url| resource_hashes.get(url))
                .copied()
                .collect::<HashSet<Hash>>(),

            packages: packages_lock_info.into_iter()
                .map(|(package_hash, (url, inputs, outputs))| {
                    let package_info = LockedPackageInfo {
                        url,

                        inputs: inputs.into_iter()
                            .flat_map(|(resource_name, (resource_url, resource_format))| {
                                let hash = resource_hashes.get(&resource_url)
                                    .copied()?;

                                let info = LockedResourceInfo {
                                    url: resource_url,
                                    format: resource_format,
                                    hash
                                };

                                Some((resource_name, info))
                            })
                            .collect(),

                        outputs: outputs.into_iter()
                            .flat_map(|(resource_name, (resource_url, resource_format))| {
                                let hash = resource_hashes.get(&resource_url)
                                    .copied()?;

                                let info = LockedResourceInfo {
                                    url: resource_url,
                                    format: resource_format,
                                    hash
                                };

                                Some((resource_name, info))
                            })
                            .collect()
                    };

                    (package_hash, package_info)
                })
                .collect(),

            resources: resources_lock_info
        })
    }
}
