pub mod manifest;
pub mod store;
pub mod generation;

pub mod prelude {
    pub use super::manifest::{
        Manifest as GenerationManifest,
        Game as GenerationGameLock
    };

    pub use super::store::{
        Store as GenerationsStore,
        StoreError as GenerationsStoreError
    };

    pub use super::generation::{
        Generation,
        GenerationError
    };
}
