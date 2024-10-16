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

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct LockFile {
    /// URLs to the root packages for the lock file.
    root_packages: HashSet<String>
}

impl LockFile {
    #[inline]
    /// Create new empty lock file.
    pub fn new() -> Self {
        Self {
            root_packages: HashSet::new(),
        }
    }

    #[inline]
    /// Create new lock file with given root packages URLs.
    pub fn with_packages<T: Into<String>>(packages: impl IntoIterator<Item = T>) -> Self {
        Self {
            root_packages: HashSet::from_iter(packages.into_iter().map(T::into)),
        }
    }

    #[inline]
    /// Add root package URL.
    pub fn add_package(&mut self, url: impl ToString) -> &mut Self {
        self.root_packages.insert(url.to_string());

        self
    }

    /// Build lock file with provided root packages URLs
    /// and a packages store.
    ///
    /// This method will download all the packages to a
    /// temporary directory, extract if needed, calculate
    /// their hashes and validate that everything is correct,
    /// returning a lock file's manifest struct.
    ///
    /// Note: Since this function will download (potentially) many
    /// files and archives you should run it in a separate thread.
    pub async fn build(&self, store: &PackagesStore) -> Result<LockFileManifest, LockFileError> {
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

        // Keep downloading stuff while we have packages to process.
        while !packages.is_empty() {
            let mut packages_contexts = Vec::with_capacity(packages.len());

            // Go through the list of packages to process.
            for (mut package_url, temp_hash, is_root) in packages.drain() {
                // Append "package.json" to the end of the URL
                // if it's missing.
                if !package_url.ends_with("/package.json") {
                    package_url += "/package.json";
                }

                // Normalize URL.
                package_url = normalize_url(package_url);

                let unique_key = (package_url.clone(), PackageResourceFormat::Package);

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

                // Start downloader task.
                let context = Downloader::new(&package_url)?
                    .with_continue_downloading(false)
                    .with_output_file(&temp_path)
                    .download(|_, _, _| {})
                    .await?;

                requested_urls.insert(unique_key.clone());
                packages_contexts.push((temp_path, package_url, root_url, unique_key, context, is_root));
            }

            let mut resources = Vec::new();

            // Go through the list of queued packages.
            for (temp_path, package_url, root_url, unique_key, context, is_root) in packages_contexts.drain(..) {
                // Await package downloading.
                context.wait()?;

                // Read the package's manifest and hash it.
                let manifest_slice = std::fs::read(&temp_path)?;
                let manifest_hash = Hash::for_slice(&manifest_slice);

                let manifest = serde_json::from_slice(&manifest_slice)?;
                let manifest = PackageManifest::from_json(&manifest)?;

                // Update the lock file info.
                let lock_resource_index = lock_resources.len();

                // Save index of the resolved package.
                resources_indexes.insert(unique_key, lock_resource_index);

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

                if is_root {
                    lock_root.insert(lock_resource_index as u64);
                }

                // Process inputs if there are some.
                if let Some(inputs) = manifest.inputs {
                    for (name, resource) in inputs {
                        let temp_hash = Hash::rand();

                        assign_references.push((temp_hash, name, lock_resource_index, true));
                        resources.push((temp_hash, root_url.clone(), resource));
                    }
                }

                // Process outputs.
                for (name, resource) in manifest.outputs {
                    let temp_hash = Hash::rand();

                    assign_references.push((temp_hash, name, lock_resource_index, false));
                    resources.push((temp_hash, root_url.clone(), resource));
                }

                // Move the manifest from the temp location to the correct one.
                let src_path = store.get_path(&manifest_hash, &PackageResourceFormat::Package);

                std::fs::rename(temp_path, src_path)?;
            }

            let mut resources_contexts = Vec::with_capacity(resources.len());

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

                // Reference temp hash to the unique key of the
                // current package. This will be used at the
                // final stage to update references in this package.
                assigned_hashes.insert(temp_hash, unique_key.clone());

                // Skip resources which already were requested.
                if requested_urls.contains(&unique_key) {
                    continue;
                }

                // If the resource is another package - queue it to the full processing.
                // Otherwise process the resource by downloading and extracting it.
                if resource.format == PackageResourceFormat::Package {
                    packages.insert((resource_url, temp_hash, false));

                    continue;
                }

                // Prepare temp path to the resource.
                let temp_path = store.get_temp_path(&temp_hash);

                // Start downloader task.
                let context = Downloader::new(&resource_url)?
                    .with_continue_downloading(false)
                    .with_output_file(&temp_path)
                    .download(|_, _, _| {})
                    .await?;

                requested_urls.insert(unique_key.clone());
                resources_contexts.push((temp_path, resource_url, unique_key, resource, context));
            }

            // Go through the list of queued resources.
            for (temp_path, resource_url, unique_key, resource, context) in resources_contexts.drain(..) {
                // Await resource downloading.
                context.wait()?;

                match resource.format {
                    PackageResourceFormat::Package => unreachable!("Package must have been queued to be processed in a different place"),

                    PackageResourceFormat::Module(_) |
                    PackageResourceFormat::File => {
                        // Move downloaded file to the correct location.
                        let hash = Hash::for_entry(&temp_path)?;
                        let src_path = store.get_path(&hash, &resource.format);

                        std::fs::rename(temp_path, &src_path)?;

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
                        let lock_resource_index = lock_resources.len();

                        // Save index of the resource.
                        resources_indexes.insert(unique_key, lock_resource_index);

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
                    }

                    PackageResourceFormat::Archive(_) => {
                        // Extract the archive to a temp folder.
                        let tmp_extract_path = store.get_temp_path(&Hash::rand());

                        match resource.format {
                            PackageResourceFormat::Archive(PackageResourceArchiveFormat::Auto) => {
                                archive_extract(&temp_path, &tmp_extract_path, |_, _, _| {})
                                    .map_err(|err| LockFileError::ExtractFailed(err.to_string()))?;
                            }

                            PackageResourceFormat::Archive(PackageResourceArchiveFormat::Tar) => {
                                TarArchive::open(&temp_path)?
                                    .extract(&tmp_extract_path, |_, _, _| {})?
                                    .wait()
                                    .map_err(|err| {
                                        LockFileError::ExtractFailed(format!("Failed to extract tar archive: {err:?}"))
                                    })?;
                            }

                            PackageResourceFormat::Archive(PackageResourceArchiveFormat::Zip) => {
                                ZipArchive::open(&temp_path)?
                                    .extract(&tmp_extract_path, |_, _, _| {})?
                                    .wait()
                                    .map_err(|err| {
                                        LockFileError::ExtractFailed(format!("Failed to extract zip archive: {err:?}"))
                                    })?;
                            }

                            PackageResourceFormat::Archive(PackageResourceArchiveFormat::Sevenz) => {
                                SevenzArchive::open(&temp_path)
                                    .and_then(|archive| archive.extract(&tmp_extract_path, |_, _, _| {}))
                                    .map(|extractor| extractor.wait())
                                    .map_err(|err| {
                                        LockFileError::ExtractFailed(format!("Failed to open 7z archive: {err:?}"))
                                    })?
                                    .map_err(|err| {
                                        LockFileError::ExtractFailed(format!("Failed to extract 7z archive: {err:?}"))
                                    })?;
                            }

                            _ => unreachable!("Non-archive format in archives-only processor")
                        }

                        // Move extracted files to the correct location
                        // and delete downloaded archive.
                        let hash = Hash::for_entry(&tmp_extract_path)?;
                        let src_path = store.get_path(&hash, &resource.format);

                        if src_path.exists() {
                            std::fs::remove_dir_all(&src_path)?;
                        }

                        std::fs::rename(tmp_extract_path, &src_path)?;
                        std::fs::remove_file(temp_path)?;

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
                        let lock_resource_index = lock_resources.len();

                        // Save index of the resource.
                        resources_indexes.insert(unique_key, lock_resource_index);

                        // Update the lock file info.
                        lock_resources.push(LockFileResourceLock {
                            url: resource_url,
                            format: resource.format,
                            lock: LockFileResourceLockData {
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
                            inputs.insert(name, *resource_index as u64);
                        }
                    } else if let Some(outputs) = &mut lock_resources[index].outputs {
                        outputs.insert(name, *resource_index as u64);
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
            root: lock_root.drain().collect(),
            resources: lock_resources
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn build() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(".agl-packages-test");

        if path.exists() {
            std::fs::remove_dir_all(&path)?;
        }

        std::fs::create_dir_all(&path)?;

        let store = PackagesStore::new(&path);

        let lock_file = LockFile::with_packages([
            "https://raw.githubusercontent.com/an-anime-team/anime-games-launcher/next/tests/packages/1"
        ]);

        let mut lock_file = lock_file.build(&store).await
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;

        assert_eq!(lock_file.root, &[0]);
        assert_eq!(lock_file.resources.len(), 8);
        assert_eq!(Hash::for_entry(path)?, Hash(9585216612201553270));

        let Some(inputs) = lock_file.resources[0].inputs.take() else {
            anyhow::bail!("No inputs in the root package");
        };

        let Some(outputs) = lock_file.resources[0].outputs.take() else {
            anyhow::bail!("No outputs in the root package");
        };

        assert_eq!(lock_file.resources[inputs["self-reference"] as usize].lock.hash, Hash(9442626994218140953));
        assert_eq!(lock_file.resources[inputs["another-package"] as usize].lock.hash, Hash(14134949798113050856));
        assert_eq!(lock_file.resources[outputs["self-reference"] as usize].lock.hash, Hash(9442626994218140953));

        Ok(())
    }
}
