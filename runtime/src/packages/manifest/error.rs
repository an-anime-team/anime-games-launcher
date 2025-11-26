#[derive(Debug, thiserror::Error)]
pub enum PackageManifestError {
    #[error("unknown resource format: {0}")]
    ResourceUnknownFormat(String),

    #[error("unknown resource module format: {0}")]
    ResourceUnknownModuleFormat(String),

    #[error("unknown resource archive format: {0}")]
    ResourceUnknownArchiveFormat(String),

    #[error("resource is missing uri field")]
    ResourceMissingUri,

    #[error("invalid resource hash format: {0}")]
    ResourceInvalidHashFormat(String),

    #[error("unknown package format version: {0}")]
    PackageUnknownFormatVersion(u16),

    #[error("invalid package manifest field '{field}' format: expected '{expected}'")]
    PackageInvalidFieldFormat {
        field: &'static str,
        expected: &'static str
    }
}
