use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::packages::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Descriptor of a resource installed in some store.
pub enum Resource {
    Package {
        /// Hash of the package's manifest.
        hash: Hash,

        /// List of package inputs with their names.
        inputs: HashMap<String, Hash>,

        /// List of package outputs with their names.
        outputs: HashMap<String, Hash>,

        /// Metadata of the package.
        metadata: Option<PackageMetadata>,

        /// Path to the package's manifest in the store.
        path: PathBuf
    },

    File {
        /// Hash of the file.
        hash: Hash,

        /// Path to the file in the store.
        path: PathBuf
    },

    Folder {
        /// Hash of the folder.
        hash: Hash,

        /// Path to the folder in the store.
        path: PathBuf
    }
}

impl Resource {
    #[inline]
    /// Get hash of the resource.
    pub fn get_hash(&self) -> &Hash {
        match self {
            Self::Package { hash, .. } |
            Self::File { hash, .. } |
            Self::Folder { hash, .. } => hash
        }
    }

    #[inline]
    /// Get path to the resource in the store.
    pub fn get_path(&self) -> &Path {
        match self {
            Self::Package { path, .. } |
            Self::File { path, .. } |
            Self::Folder { path, .. } => path
        }
    }

    #[inline]
    /// Check if the current resource is a package.
    pub fn is_package(&self) -> bool {
        matches!(self, Self::Package { .. })
    }

    #[inline]
    /// Get metadata of the current package.
    ///
    /// Returns none if the current resource is not a package.
    pub fn get_metadata(&self) -> Option<&PackageMetadata> {
        if let Self::Package { metadata, .. } = self {
            metadata.as_ref()
        } else {
            None
        }
    }

    #[inline]
    /// Get table of inputs for the current package.
    ///
    /// Returns none if the current resource is not a package.
    pub fn get_inputs(&self) -> Option<&HashMap<String, Hash>> {
        if let Self::Package { inputs, .. } = self {
            Some(inputs)
        } else {
            None
        }
    }

    #[inline]
    /// Get table of outputs for the current package.
    ///
    /// Returns none if the current resource is not a package.
    pub fn get_outputs(&self) -> Option<&HashMap<String, Hash>> {
        if let Self::Package { outputs, .. } = self {
            Some(outputs)
        } else {
            None
        }
    }
}
