pub mod manifest;
pub mod registry;
pub mod engine;

pub mod prelude {
    pub use super::manifest::GameManifest;
    pub use super::manifest::localizable_string::LocalizableString;
    pub use super::registry::Manifest as GamesRegistryManifest;

    pub use super::engine::{
        GameEngine,
        GameEdition,
        GameComponent,
        GameLaunchInfo,
        InstallationStatus,
        InstallationDiff
    };
}
