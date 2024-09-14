use std::collections::{HashMap, HashSet};

use crate::core::prelude::*;
use crate::packages::prelude::*;

pub mod manifest;

#[derive(Debug, thiserror::Error)]
pub enum LockFileError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Failed to deserialize package manifest: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("Failed to decode package manifest: {0}")]
    AsJson(#[from] AsJsonError),

    #[error("Failed to download package: {0}")]
    Downloader(#[from] DownloaderError),

    #[error("Failed to extract archive: {0}")]
    ExtractFailed(String),

    #[error("Resource hash mismatch. Current: {}, expected: {}", current.to_base32(), expected.to_base32())]
    HashMismatch {
        current: Hash,
        expected: Hash
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockFile {
    /// Store used to verify which packages are
    /// already installed and to which the packages
    /// will be installed during building of the
    /// lock file.
    store: Store,

    /// URLs to the root packages for the lock file.
    root_packages: HashSet<String>,
}

impl LockFile {
    #[inline]
    /// Create new empty lock file.
    pub fn new(store: Store) -> Self {
        Self {
            store,
            root_packages: HashSet::new(),
        }
    }

    #[inline]
    /// Create new lock file with given root packages URLs.
    pub fn with_packages(store: Store, packages: impl IntoIterator<Item = String>) -> Self {
        Self {
            store,
            root_packages: HashSet::from_iter(packages),
        }
    }

    #[inline]
    /// Add root package URL.
    pub fn add_package(&mut self, url: impl ToString) -> &mut Self {
        self.root_packages.insert(url.to_string());

        self
    }

    /// Build lock file with provided root packages URLs.
    ///
    /// This method will download all the packages to a
    /// given temporary directory, extract if needed,
    /// calculate their hashes and validate that everything
    /// is correct, returning a lock file's manifest struct.
    ///
    /// Since this function will download (potentially) many
    /// files and archives you should run it in a separate thread.
    pub async fn build(&self) -> Result<LockFileManifest, LockFileError> {
        let mut packages = self.root_packages.clone();
        let mut lock_resources = Vec::with_capacity(packages.len());
        let mut requested_urls = HashSet::with_capacity(packages.len());

        // Keep downloading stuff while we have packages to process.
        while !packages.is_empty() {
            let mut packages_contexts = Vec::with_capacity(packages.len());

            // Go through the list of packages to process.
            for mut package_url in packages.drain() {
                // Append "package.json" to the end of the URL
                // if it's missing.
                if !package_url.ends_with("/package.json") {
                    package_url += "/package.json";
                }

                // Skip packages which already were requested.
                if requested_urls.contains(&package_url) {
                    continue;
                }

                // Prepare URL to the package's root folder.
                let root_url = package_url
                    .strip_suffix("package.json")
                    .map(String::from)
                    .unwrap_or_else(|| package_url.clone());

                // Prepare tmp path to the package.
                let tmp_path = self.store.folder()
                    .join(format!("{}.tmp", Hash::rand().to_base32()));

                // Start downloader task.
                let context = Downloader::new(&package_url)?
                    .with_continue_downloading(false)
                    .with_output_file(&tmp_path)
                    .download(|_, _, _| {})
                    .await?;

                requested_urls.insert(package_url.clone());
                packages_contexts.push((tmp_path, package_url, root_url, context));
            }

            let mut resources = Vec::new();

            // Go through the list of queued packages.
            for (tmp_path, package_url, root_url, context) in packages_contexts.drain(..) {
                // Await package downloading.
                context.wait()?;

                // Read the package's manifest and hash it.
                let manifest_slice = std::fs::read(&tmp_path)?;
                let manifest = serde_json::from_slice(&manifest_slice)?;

                let manifest = PackageManifest::from_json(&manifest)?;
                let manifest_hash = Hash::for_slice(&manifest_slice);

                // Update the lock file info.
                let lock_resource_index = lock_resources.len();

                lock_resources.push(LockFileResourceLock {
                    url: package_url,
                    format: PackageResourceFormat::Package,
                    lock: LockFileResourceLockData {
                        hash: manifest_hash,
                        size: manifest_slice.len() as u64
                    },
                    inputs: Some({
                        match &manifest.inputs {
                            Some(inputs) => HashMap::with_capacity(inputs.len()),
                            None => HashMap::new()
                        }
                    }),
                    outputs: Some(HashMap::with_capacity(manifest.outputs.len()))
                });

                // Process inputs if there are some.
                if let Some(inputs) = manifest.inputs {
                    for (name, resource) in inputs {
                        resources.push((root_url.clone(), resource, name, lock_resource_index, true));
                    }
                }

                // Process outputs.
                for (name, resource) in manifest.outputs {
                    resources.push((root_url.clone(), resource, name, lock_resource_index, false));
                }

                // Move the manifest from the temp location to the correct one.
                let src_path = self.store.folder()
                    .join(format!("{}.src", manifest_hash.to_base32()));

                std::fs::rename(tmp_path, src_path)?;
            }

            let mut resources_contexts = Vec::with_capacity(resources.len());

            // Go through the list of packages' resources to process.
            for (root_url, resource, lock_resource_name, lock_resource_index, is_input) in resources.drain(..) {
                // Skip resource processing if it's already installed.
                if let Some(hash) = resource.hash {
                    // Do not check for packages because their dependencies
                    // could be updated upstream.
                    if self.store.has_resource(&hash) {
                        if is_input {
                            if let Some(inputs) = &mut lock_resources[lock_resource_index].inputs {
                                inputs.insert(lock_resource_name, hash);
                            }
                        } else if let Some(outputs) = &mut lock_resources[lock_resource_index].outputs {
                            outputs.insert(lock_resource_name, hash);
                        }

                        continue;
                    }
                }

                // Prepare URL to the resource.
                let resource_url = if resource.uri.starts_with("http") {
                    resource.uri.clone()
                } else {
                    format!("{root_url}/{}", resource.uri)
                };

                // Skip resources which already were requested.
                if requested_urls.contains(&resource_url) {
                    continue;
                }

                // If the resource is another package - queue it to the full processing.
                // Otherwise process the resource by downloading and extracting it.
                if resource.format == PackageResourceFormat::Package {
                    packages.insert(resource_url);

                    continue;
                }

                // Prepare temp path to the resource.
                let resource_hash = resource.hash
                    .unwrap_or(Hash::rand())
                    .to_base32();

                let tmp_path = self.store.folder()
                    .join(format!("{resource_hash}.tmp"));

                // Start downloader task.
                let context = Downloader::new(&resource_url)?
                    .with_continue_downloading(false)
                    .with_output_file(&tmp_path)
                    .download(|_, _, _| {})
                    .await?;

                requested_urls.insert(resource_url.clone());
                resources_contexts.push((tmp_path, resource_url, resource, context, lock_resource_name, lock_resource_index, is_input));
            }

            // Go through the list of queued resources.
            for (tmp_path, resource_url, resource, context, lock_resource_name, lock_resource_index, is_input) in resources_contexts.drain(..) {
                // Await resource downloading.
                context.wait()?;

                match resource.format {
                    PackageResourceFormat::Package => unreachable!("Package must have been queued to be processed in a different place"),

                    PackageResourceFormat::File => {
                        // Move downloaded file to the correct location.
                        let hash = Hash::for_entry(&tmp_path)?;

                        let src_path = self.store.folder()
                            .join(hash.to_base32());

                        std::fs::rename(tmp_path, &src_path)?;

                        // Verify hashes match.
                        if let Some(expected_hash) = resource.hash {
                            if expected_hash != hash {
                                return Err(LockFileError::HashMismatch {
                                    current: hash,
                                    expected: expected_hash
                                });
                            }
                        }

                        // Update the lock file info.
                        lock_resources.push(LockFileResourceLock {
                            url: resource_url,
                            format: resource.format,
                            lock: LockFileResourceLockData {
                                hash,
                                size: src_path.metadata()?.len()
                            },
                            inputs: None,
                            outputs: None
                        });

                        if is_input {
                            if let Some(inputs) = &mut lock_resources[lock_resource_index].inputs {
                                inputs.insert(lock_resource_name, hash);
                            }
                        } else if let Some(outputs) = &mut lock_resources[lock_resource_index].outputs {
                            outputs.insert(lock_resource_name, hash);
                        }
                    }

                    PackageResourceFormat::Archive |
                    PackageResourceFormat::Tar |
                    PackageResourceFormat::Zip |
                    PackageResourceFormat::Sevenz => {
                        // Extract the archive to a temp folder.
                        let tmp_extract_path = self.store.folder()
                            .join(format!("{}.tmp", Hash::rand().to_base32()));

                        match resource.format {
                            PackageResourceFormat::Archive => {
                                archive_extract(&tmp_path, &tmp_extract_path, |_, _, _| {})
                                    .map_err(|err| LockFileError::ExtractFailed(err.to_string()))?;
                            }

                            PackageResourceFormat::Tar => {
                                TarArchive::open(&tmp_path)?
                                    .extract(&tmp_extract_path, |_, _, _| {})?
                                    .wait()
                                    .map_err(|err| {
                                        LockFileError::ExtractFailed(format!("Failed to extract tar archive: {err:?}"))
                                    })?;
                            }

                            PackageResourceFormat::Zip => {
                                ZipArchive::open(&tmp_path)?
                                    .extract(&tmp_extract_path, |_, _, _| {})?
                                    .wait()
                                    .map_err(|err| {
                                        LockFileError::ExtractFailed(format!("Failed to extract zip archive: {err:?}"))
                                    })?;
                            }

                            PackageResourceFormat::Sevenz => {
                                SevenzArchive::open(&tmp_path)
                                    .and_then(|archive| archive.extract(&tmp_extract_path, |_, _, _| {}))
                                    .map(|extractor| extractor.wait())
                                    .map_err(|err| {
                                        LockFileError::ExtractFailed(format!("Failed to open 7z archive: {err:?}"))
                                    })?
                                    .map_err(|err| {
                                        LockFileError::ExtractFailed(format!("Failed to extract 7z archive: {err:?}"))
                                    })?;
                            }

                            _ => unreachable!()
                        }

                        // Move extracted files to the correct location
                        // and delete downloaded archive.
                        let hash = Hash::for_entry(&tmp_extract_path)?;

                        let src_path = self.store.folder()
                            .join(hash.to_base32());

                        if src_path.exists() {
                            std::fs::remove_dir_all(&src_path)?;
                        }

                        std::fs::rename(tmp_extract_path, &src_path)?;
                        std::fs::remove_file(tmp_path)?;

                        // Verify hashes match.
                        if let Some(expected_hash) = resource.hash {
                            if expected_hash != hash {
                                return Err(LockFileError::HashMismatch {
                                    current: hash,
                                    expected: expected_hash
                                });
                            }
                        }

                        // Update the lock file info.
                        lock_resources.push(LockFileResourceLock {
                            url: resource_url,
                            format: resource.format,
                            lock: LockFileResourceLockData {
                                hash,

                                // TODO: might be incorrect. Better sum lengths of
                                // all the files inside the folder.
                                size: src_path.metadata()?.len()
                            },
                            inputs: None,
                            outputs: None
                        });

                        if is_input {
                            if let Some(inputs) = &mut lock_resources[lock_resource_index].inputs {
                                inputs.insert(lock_resource_name, hash);
                            }
                        } else if let Some(outputs) = &mut lock_resources[lock_resource_index].outputs {
                            outputs.insert(lock_resource_name, hash);
                        }
                    }
                }
            }
        }

        let generated_at = std::time::UNIX_EPOCH.elapsed()
            .unwrap_or_default()
            .as_secs();

        Ok(LockFileManifest {
            standard: 1,
            metadata: LockFileMetadata {
                generated_at
            },
            root: vec![],
            resources: lock_resources
        })
    }
}
