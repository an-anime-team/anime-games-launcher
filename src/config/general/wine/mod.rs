use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

pub mod wine_sync;
pub mod wine_lang;
pub mod virtual_desktop;
pub mod shared_libraries;

pub mod prelude {
    pub use super::wine_sync::WineSync;
    pub use super::wine_lang::WineLang;
    pub use super::virtual_desktop::VirtualDesktop;
    pub use super::shared_libraries::SharedLibraries;
    pub use super::Wine;
}

use prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Wine {
    pub sync: WineSync,
    pub language: WineLang,
    pub borderless: bool,
    pub virtual_desktop: VirtualDesktop,
    pub shared_libraries: SharedLibraries
}

impl Default for Wine {
    #[inline]
    fn default() -> Self {
        Self {
            sync: WineSync::default(),
            language: WineLang::default(),
            borderless: false,
            virtual_desktop: VirtualDesktop::default(),
            shared_libraries: SharedLibraries::default()
        }
    }
}

impl From<&Json> for Wine {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            sync: value.get("sync")
                .map(WineSync::from)
                .unwrap_or(default.sync),

            language: value.get("language")
                .map(WineLang::from)
                .unwrap_or(default.language),

            borderless: value.get("borderless")
                .and_then(Json::as_bool)
                .unwrap_or(default.borderless),

            virtual_desktop: value.get("virtual_desktop")
                .map(VirtualDesktop::from)
                .unwrap_or(default.virtual_desktop),

            shared_libraries: value.get("shared_libraries")
                .map(SharedLibraries::from)
                .unwrap_or(default.shared_libraries)
        }
    }
}
