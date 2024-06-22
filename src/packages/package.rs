use std::path::PathBuf;

use serde_json::Value as Json;

use super::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    uri: String,
    manifest: Manifest,
    plain_manifest: Vec<u8>
}

impl Package {
    pub async fn fetch(uri: impl ToString) -> anyhow::Result<Self> {
        let uri = uri.to_string();

        let plain_manifest = crate::handlers::handle(format!("{uri}/manifest.json"))?
            .join().await?
            .map_err(|err| anyhow::anyhow!("Failed to request package's manifest file: {err}"))?;

        let manifest = serde_json::from_slice::<Json>(&plain_manifest)?;

        let Some(manifest_version) = manifest.get("manifest_version") else {
            anyhow::bail!("Incorrect manifest file format: `manifest_version` field is missing");
        };

        // Compatibility with v1 format
        let manifest_version = manifest_version
            .as_u64()
            .or_else(|| {
                manifest_version
                    .as_str()
                    .and_then(|version| {
                        version.parse::<u64>().ok()
                    })
            })
            .ok_or_else(|| anyhow::anyhow!("Incorrect manifest file format: `manifest_version` field is incorrect"))?;

        let manifest = match manifest_version {
            1 => parse_v1(&manifest, uri.clone()).await?,
            2 => parse_v2(&manifest)?,

            _ => anyhow::bail!("Incorrect manifest file format: unsupported manifest version: {manifest_version}")
        };

        Ok(Self {
            uri,
            manifest,
            plain_manifest
        })
    }

    #[inline]
    /// Get package's base URI
    pub fn uri(&self) -> &str {
        &self.uri
    }

    #[inline]
    /// Get package's manifest
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    #[inline]
    /// Get package's plain manifest
    pub fn plain_manifest(&self) -> &[u8] {
        &self.plain_manifest
    }

    #[inline]
    /// Calculate hash of the package's manifest
    pub fn get_manifest_hash(&self) -> Hash {
        Hash::from_slice(HashAlgorithm::default(), &self.plain_manifest)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageInput {
    Package(Package),
    File(PathBuf),
    ZipArchive(PathBuf),
    TarArchive(PathBuf),
    TarGzArchive(PathBuf)
}
