pub mod game;
pub mod addons;
pub mod diff;
pub mod download;
pub mod integrity;

pub mod prelude {
    pub use super::game::{
        Edition as GameEdition,
        Status as GameStatus,
        LaunchOptions as GameLaunchOptions
    };

    pub use super::addons::*;
    pub use super::diff::*;
    pub use super::download::*;
    pub use super::integrity::*;

    pub use super::IntegrationStandard;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntegrationStandard {
    V1
}
