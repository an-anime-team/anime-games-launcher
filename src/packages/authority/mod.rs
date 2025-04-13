use serde_json::Value as Json;

pub mod manifest;

use crate::prelude::*;

#[derive(Debug, thiserror::Error)]
pub enum AuthorityValidatorError {
    #[error(transparent)]
    Tokio(#[from] tokio::task::JoinError),

    #[error("Failed to fetch authority index manifest: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Failed to deserialize authority index manifest: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("Failed to decode authority index manifest: {0}")]
    AsJson(#[from] AsJsonError)
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
/// Special struct to keep track of different authority indexes manifests
/// and request merged information about packages.
pub struct AuthorityValidator {
    manifests: Vec<manifest::Manifest>
}

impl AuthorityValidator {
    /// Create new validator from provided list of manifests.
    #[inline]
    pub fn new(manifests: impl IntoIterator<Item = manifest::Manifest>) -> Self {
        Self {
            manifests: manifests.into_iter().collect()
        }
    }

    /// Fetch authority indexes from provided urls.
    pub async fn build<T: ToString>(urls: impl IntoIterator<Item = T>) -> Result<Self, AuthorityValidatorError> {
        let client = STARTUP_CONFIG.general.network.builder()?.build()?;

        let mut responses = Vec::new();

        // Enqueue manifests fetching.
        for url in urls {
            let response = tokio::spawn(client.get(url.to_string()).send());

            responses.push(response);
        }

        let mut manifests = Vec::with_capacity(responses.len());

        // Iterate over fetch responses.
        for response in responses.drain(..) {
            let manifest = response.await??;

            manifests.push(tokio::spawn(manifest.bytes()));
        }

        let mut result = Vec::with_capacity(manifests.len());

        // Iterate over fetched manifests.
        for manifest in manifests.drain(..) {
            let manifest = manifest.await??;

            let manifest = serde_json::from_slice::<Json>(&manifest)?;
            let manifest = manifest::Manifest::from_json(&manifest)?;

            tracing::trace!(
                title = manifest.title.default_translation(),
                "Fetched authority index"
            );

            result.push(manifest);
        }

        Ok(Self {
            manifests: result
        })
    }

    /// Search for the first ocurrence of the resource information within
    /// the stored authority indexes.
    ///
    /// ! Note that this should not be used to determine resource's status.
    /// ! This method must be used to retrieve its general information only.
    /// ! Use `get_status` to acquire proper status instead.
    pub fn lookup(&self, hash: &Hash) -> Option<&manifest::ResourceInfo> {
        for manifest in &self.manifests {
            for resource in &manifest.resources {
                for variant in &resource.variants {
                    if variant.contains(hash) {
                        return Some(resource);
                    }
                }
            }
        }

        None
    }

    /// Read all the stored authority indexes and merge their statuses about
    /// the requested resource.
    ///
    /// Safe method for retrieving resource's status. Will immediately return
    /// malicious or compromised resources if any found.
    pub fn get_status(&self, hash: &Hash) -> Option<manifest::ResourceStatus> {
        let mut status = None;

        for manifest in &self.manifests {
            for resource in &manifest.resources {
                for variant in &resource.variants {
                    if variant.contains(hash) {
                        match (&mut status, variant.to_owned()) {
                            // Immediately return status if the resource is marked
                            // as malicious or compromised.
                            (_, resource_status) if matches!(resource_status, manifest::ResourceStatus::Compromised { .. }) => return Some(resource_status),
                            (_, resource_status) if matches!(resource_status, manifest::ResourceStatus::Malicious { .. }) => return Some(resource_status),

                            // Merge information about trusted resource status.
                            (Some(manifest::ResourceStatus::Trusted {
                                ext_process_api: curr_ext_process_api,
                                allowed_paths: curr_allowed_paths,
                                hashes: curr_hashes
                            }), manifest::ResourceStatus::Trusted {
                                ext_process_api,
                                allowed_paths,
                                hashes
                            }) => {
                                if curr_ext_process_api.is_none() {
                                    *curr_ext_process_api = ext_process_api;
                                }

                                // If at least one authority index *implicitly* disabled process API
                                // we *should* disable it here as a safety guaranty.
                                else if ext_process_api == Some(false) {
                                    *curr_ext_process_api = Some(false);
                                }

                                if let Some(allowed_paths) = allowed_paths {
                                    match curr_allowed_paths {
                                        Some(curr_allowed_paths) => {
                                            curr_allowed_paths.extend(allowed_paths);
                                        }

                                        None => *curr_allowed_paths = Some(allowed_paths)
                                    }
                                }

                                curr_hashes.extend(hashes);
                            }

                            // Set information if it's not set yet.
                            (None, resource_status) => status = Some(resource_status),

                            _ => unreachable!()
                        }
                    }
                }
            }
        }

        status
    }
}
