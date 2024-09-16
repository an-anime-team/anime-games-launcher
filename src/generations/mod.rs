use std::path::PathBuf;

pub mod manifest;

pub mod prelude {
    pub use super::manifest::{
        Manifest as GenerationManifest,
        Game as GenerationGameLock
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generations {
    folder: PathBuf
}

impl Generations {
    #[inline]
    /// Create new empty generations store.
    pub fn new(folder: impl Into<PathBuf>) -> Self {
        Self {
            folder: folder.into()
        }
    }
}
