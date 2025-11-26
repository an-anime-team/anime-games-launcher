use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use wineyard_core::network::downloader::{
    Downloader, DownloadOptions, DownloaderError
};
use wineyard_core::archives::{Archive, ArchiveFormat, ArchiveError};

use toml::Table as TomlTable;

use crate::hash::Hash;

use super::manifest::{
    PackageManifest, PackageManifestError, ResourceFormat, ResourceArchiveFormat
};
use super::lock_file::{
    LockFile, LockFileInfo, ResourceLock, ResourceLockData, LockFileError
};
use super::store::ResourceStore;

#[derive(Debug, thiserror::Error)]
pub enum PackagesResolverError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    DownloaderError(#[from] DownloaderError),

    #[error("archive format is not supported: {0:?}")]
    ArchiveNotSupported(PathBuf),

    #[error(transparent)]
    ArchiveError(#[from] ArchiveError),

    #[error(transparent)]
    LockFileError(#[from] LockFileError),

    #[error(transparent)]
    PackageManifestError(#[from] PackageManifestError),

    #[error(transparent)]
    Serialize(#[from] toml::de::Error),

    #[error("resource with hash {current} was expected to have hash {expected}")]
    HashMismatch {
        current: String,
        expected: String
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct PackagesResolver {
    /// URLs to the root packages for the lock file.
    root_packages: HashSet<String>
}

impl PackagesResolver {
    /// Create new empty lock file.
    #[inline]
    pub fn new() -> Self {
        Self {
            root_packages: HashSet::new()
        }
    }

    /// Create new lock file with given root packages URLs.
    #[inline]
    pub fn with_packages<T: ToString>(packages: impl IntoIterator<Item = T>) -> Self {
        let packages = packages.into_iter()
            .map(|package| package.to_string());

        Self {
            root_packages: HashSet::from_iter(packages)
        }
    }

    /// Add root package URL.
    #[inline]
    pub fn add_package(&mut self, url: impl ToString) -> &mut Self {
        self.root_packages.insert(url.to_string());

        self
    }

    /// Build lock file with provided root packages URLs and a packages store.
    ///
    /// This method will download all the packages to a temporary directory,
    /// extract if needed, calculate their hashes and validate that everything
    /// is correct, returning a lock file's manifest struct.
    ///
    /// Note: Since this function will download (potentially) many files and
    /// archives you should run it in a separate thread.
    pub async fn build(&self, store: &ResourceStore) -> Result<LockFile, PackagesResolverError> {
        let mut packages = self.root_packages.iter()
            .cloned()
            .map(|url| (url, Hash::rand(), true))
            .collect::<HashSet<_>>();

        // Lock file building logic.
        let mut lock_resources = Vec::with_capacity(packages.len());
        let mut lock_root = HashSet::with_capacity(packages.len());

        // Loops and self-references prevention logic.
        let mut requested_urls = HashSet::with_capacity(packages.len());

        // Inputs/outputs references updates logic.
        let mut resources_indexes = HashMap::new(); // unique_key => resource_index
        let mut assigned_hashes = HashMap::new(); // temp_hash => unique_key
        let mut assign_references = Vec::new(); // temp_hash => index to assign

        #[inline]
        /// Normalize given URL.
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

        // Prepare packages downloader.
        let downloader = Downloader::new();

        // Keep downloading stuff while we have packages to process.
        while !packages.is_empty() {
            let mut packages_download_tasks = Vec::with_capacity(packages.len());

            // Go through the list of packages to process.
            for (mut package_url, temp_hash, is_root) in packages.drain() {
                // Append "package.json" to the end of the URL
                // if it's missing.
                if !package_url.ends_with("/package.json") {
                    package_url += "/package.json";
                }

                // Normalize URL.
                package_url = normalize_url(package_url);

                let unique_key = (package_url.clone(), ResourceFormat::Package);

                // Reference temp hash to the unique key of the
                // current package. This will be used at the
                // final stage to update references in this package.
                assigned_hashes.insert(temp_hash, unique_key.clone());

                // Skip packages which already were requested.
                if requested_urls.contains(&unique_key) {
                    continue;
                }

                // Prepare URL to the package's root folder.
                let root_url = package_url
                    .strip_suffix("package.json")
                    .map(String::from)
                    .unwrap_or_else(|| package_url.clone());

                // Prepare tmp path to the package.
                let temp_path = store.get_temp_path(&temp_hash);

                // Start downloading the package.
                let task = downloader.download_with_options(&package_url, &temp_path, DownloadOptions {
                    continue_download: false,
                    on_update: None,
                    on_finish: None
                });

                requested_urls.insert(unique_key.clone());
                packages_download_tasks.push((temp_path, package_url, root_url, unique_key, task, is_root));
            }

            let mut resources = Vec::new();

            // Go through the list of queued packages.
            for (temp_path, package_url, root_url, unique_key, context, is_root) in packages_download_tasks.drain(..) {
                // Await package downloading.
                context.wait().await?;

                // Read the package's manifest and hash it.
                let manifest_slice = std::fs::read_to_string(&temp_path)?;
                let manifest_hash = Hash::for_slice(manifest_slice.as_bytes());

                let manifest = toml::from_str::<TomlTable>(&manifest_slice)?;
                let manifest = PackageManifest::try_from(&manifest)?;

                // Update the lock file info.
                let lock_resource_index = lock_resources.len();

                // Save index of the resolved package.
                resources_indexes.insert(unique_key, lock_resource_index);

                lock_resources.push(ResourceLock {
                    url: package_url,
                    format: ResourceFormat::Package,
                    lock: ResourceLockData {
                        hash: manifest_hash,
                        size: manifest_slice.len() as u64
                    },
                    inputs: Some(HashMap::with_capacity(manifest.inputs.len())),
                    outputs: Some(HashMap::with_capacity(manifest.outputs.len()))
                });

                if is_root {
                    lock_root.insert(lock_resource_index as u32);
                }

                // Process inputs.
                for (name, resource) in manifest.inputs {
                    let temp_hash = Hash::rand();

                    assign_references.push((temp_hash, name, lock_resource_index, true));
                    resources.push((temp_hash, root_url.clone(), resource));
                }

                // Process outputs.
                for (name, resource) in manifest.outputs {
                    let temp_hash = Hash::rand();

                    assign_references.push((temp_hash, name, lock_resource_index, false));
                    resources.push((temp_hash, root_url.clone(), resource));
                }

                // Move the manifest from the temp location to the correct one.
                let src_path = store.get_path(&manifest_hash);

                std::fs::rename(temp_path, src_path)?;
            }

            let mut resources_download_tasks = Vec::with_capacity(resources.len());

            // Go through the list of packages' resources to process.
            for (temp_hash, root_url, resource) in resources.drain(..) {
                // Skip resource processing if it's already installed.
                if let Some(hash) = resource.hash {
                    // Do not check for packages because their dependencies
                    // could be updated upstream.
                    if store.has_resource(&hash) {
                        continue;
                    }
                }

                // Prepare URL to the resource.
                let mut resource_url = if resource.uri.starts_with("http") {
                    resource.uri.clone()
                } else {
                    format!("{root_url}/{}", resource.uri)
                };

                // Normalize URL.
                resource_url = normalize_url(resource_url);

                let unique_key = (resource_url.clone(), resource.format);

                // Reference temp hash to the unique key of the current package.
                // This will be used at the final stage to update references in
                // this package.
                assigned_hashes.insert(temp_hash, unique_key.clone());

                // Skip resources which already were requested.
                if requested_urls.contains(&unique_key) {
                    continue;
                }

                // If the resource is another package - queue it to the full
                // processing. Otherwise process the resource by downloading and
                // extracting it.
                if resource.format == ResourceFormat::Package {
                    packages.insert((resource_url, temp_hash, false));

                    continue;
                }

                // Prepare temp path to the resource.
                let temp_path = store.get_temp_path(&temp_hash);

                // Start resource downloading.
                let task = downloader.download_with_options(&resource_url, &temp_path, DownloadOptions {
                    continue_download: false,
                    on_update: None,
                    on_finish: None
                });

                requested_urls.insert(unique_key.clone());
                resources_download_tasks.push((temp_path, resource_url, unique_key, resource, task));
            }

            // Go through the list of queued resources.
            for (temp_path, resource_url, unique_key, resource, context) in resources_download_tasks.drain(..) {
                // Await resource downloading.
                context.wait().await?;

                match resource.format {
                    ResourceFormat::Package => unreachable!("package must have been queued to be processed in a different place"),

                    ResourceFormat::Module(_) |
                    ResourceFormat::File => {
                        // Move downloaded file to the correct location.
                        let hash = Hash::for_entry(&temp_path)?;
                        let src_path = store.get_path(&hash);

                        std::fs::rename(temp_path, &src_path)?;

                        // Verify hashes match.
                        if let Some(expected_hash) = resource.hash {
                            if expected_hash != hash {
                                return Err(PackagesResolverError::HashMismatch {
                                    current: hash.to_base32(),
                                    expected: expected_hash.to_base32()
                                });
                            }
                        }

                        // Update the lock file info.
                        let lock_resource_index = lock_resources.len();

                        // Save index of the resource.
                        resources_indexes.insert(unique_key, lock_resource_index);

                        // Update the lock file info.
                        lock_resources.push(ResourceLock {
                            url: resource_url,
                            format: resource.format,
                            lock: ResourceLockData {
                                hash,
                                size: src_path.metadata()?.len()
                            },
                            inputs: None,
                            outputs: None
                        });
                    }

                    ResourceFormat::Archive(_) => {
                        // Extract the archive to a temp folder.
                        let temp_extract_path = store.get_temp_path(&Hash::rand());

                        let format = match resource.format {
                            ResourceFormat::Archive(ResourceArchiveFormat::Auto) => None,

                            ResourceFormat::Archive(ResourceArchiveFormat::Tar) => Some(ArchiveFormat::Tar),
                            ResourceFormat::Archive(ResourceArchiveFormat::Zip) => Some(ArchiveFormat::Zip),
                            ResourceFormat::Archive(ResourceArchiveFormat::Sevenz) => Some(ArchiveFormat::Sevenz),

                            _ => unreachable!("non-archive format in archives-only processor")
                        };

                        let archive = match format {
                            Some(format) => Archive::open_with_format(&temp_path, format),
                            None => Archive::open(&temp_path)
                        };

                        archive
                            .ok_or_else(|| PackagesResolverError::ArchiveNotSupported(temp_path.clone()))?
                            .extract(&temp_extract_path)?
                            .wait()?;

                        // Move extracted files to the correct location
                        // and delete downloaded archive.
                        let hash = Hash::for_entry(&temp_extract_path)?;
                        let src_path = store.get_path(&hash);

                        if src_path.exists() {
                            std::fs::remove_dir_all(&src_path)?;
                        }

                        std::fs::rename(temp_extract_path, &src_path)?;
                        std::fs::remove_file(temp_path)?;

                        // Verify hashes match.
                        if let Some(expected_hash) = resource.hash {
                            if expected_hash != hash {
                                return Err(PackagesResolverError::HashMismatch {
                                    current: hash.to_base32(),
                                    expected: expected_hash.to_base32()
                                });
                            }
                        }

                        // Update the lock file info.
                        let lock_resource_index = lock_resources.len();

                        // Save index of the resource.
                        resources_indexes.insert(unique_key, lock_resource_index);

                        // Update the lock file info.
                        lock_resources.push(ResourceLock {
                            url: resource_url,
                            format: resource.format,
                            lock: ResourceLockData {
                                hash,

                                // FIXME: incorrect! sum lengths of all the files.
                                size: src_path.metadata()?.len()
                            },
                            inputs: None,
                            outputs: None
                        });
                    }
                }
            }
        }

        // Update packages' inputs/outputs references.
        for (temp_hash, name, index, is_input) in assign_references {
            // Resolve unique key of the temp hash.
            if let Some(unique_key) = assigned_hashes.get(&temp_hash) {
                // Resolve actual hash of the resource by its unique key.
                if let Some(resource_index) = resources_indexes.get(unique_key) {
                    // Update the package's reference.
                    if is_input {
                        if let Some(inputs) = &mut lock_resources[index].inputs {
                            inputs.insert(name, *resource_index as u32);
                        }
                    } else if let Some(outputs) = &mut lock_resources[index].outputs {
                        outputs.insert(name, *resource_index as u32);
                    }
                }
            }
        }

        Ok(LockFile {
            lock: LockFileInfo {
                root: lock_root.drain().collect()
            },
            resources: lock_resources
        })
    }
}
