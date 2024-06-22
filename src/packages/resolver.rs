use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;

use super::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dependency {
    Input {
        name: String,
        input: ManifestInput,
        uri: String,
        manifest: Vec<u8>
    },

    Output {
        name: String,
        output: ManifestOutput,
        uri: String,
        manifest: Vec<u8>
    }
}

impl Dependency {
    #[inline]
    /// Get dependency name
    pub fn name(&self) -> &str {
        match self {
            Self::Input { name, .. } |
            Self::Output { name, .. } => name
        }
    }

    #[inline]
    /// Get dependency source URI
    pub fn uri(&self) -> &str {
        match self {
            Self::Input { uri, .. } |
            Self::Output { uri, .. } => uri
        }
    }

    #[inline]
    /// Get dependency manifest
    pub fn manifest(&self) -> &[u8] {
        match self {
            Self::Input { manifest, .. } |
            Self::Output { manifest, .. } => manifest
        }
    }

    #[inline]
    /// Check if current dependency is an input
    pub fn is_input(&self) -> bool {
        matches!(self, Self::Input { .. })
    }

    #[inline]
    /// Check if current dependency is an is_output
    pub fn is_output(&self) -> bool {
        matches!(self, Self::Output { .. })
    }

    #[inline]
    /// Resolve current package in the given storage
    pub async fn resolve(&self, storage: &Storage) -> anyhow::Result<Option<PathBuf>> {
        match self {
            Self::Input { input, .. } => Ok(storage.search_hash(&input.hash).await?),
            Self::Output { output, .. } => Ok(storage.search_hash(&output.hash).await?)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Resolver;

impl Resolver {
    /// Resolve dependency tree of the given package
    pub async fn resolve_dependencies(package: Package) -> anyhow::Result<HashSet<Dependency>> {
        let mut packages_queue = VecDeque::from([package]);
        let mut dependencies = HashSet::new();

        const PACKAGES_QUEUE_MAX_DEPTH: u8 = 32;

        let mut depth = 1;

        // Iterate over the packages queue
        while let Some(package) = packages_queue.pop_front() {
            // Go through package's inputs
            for (name, input) in package.manifest().inputs.clone() {
                // Resolve input if it's a package and put it to the queue
                if input.format.is_package() {
                    let package = Package::fetch(&input.uri).await?;

                    // FIXME: we can face an infinite loop here on cyclic references
                    packages_queue.push_back(package);
                }

                dependencies.insert(Dependency::Input {
                    uri: input.uri.clone(),
                    manifest: package.plain_manifest().to_vec(),
                    name,
                    input
                });
            }

            // Go through the package's outputs
            for (name, output) in package.manifest().outputs.clone() {
                dependencies.insert(Dependency::Output {
                    uri: package.uri().to_string(),
                    manifest: package.plain_manifest().to_vec(),
                    name,
                    output
                });
            }

            depth += 1;

            // Current workaround for possible cyclic references
            // Should be replaced in future by some smarter hashes comparison mechanism
            if depth >= PACKAGES_QUEUE_MAX_DEPTH {
                break;
            }
        }

        Ok(dependencies)
    }
}
