use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

pub mod hud;
pub mod fsr;

pub mod prelude {
    pub use super::hud::HUD;
    pub use super::fsr::{
        FSR,
        FsrQuality
    };

    pub use super::Enhancements;
}

use prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enhancements {
    pub hud: HUD,
    pub fsr: FSR,
    pub gamemode: bool
}

impl Default for Enhancements {
    #[inline]
    fn default() -> Self {
        Self {
            hud: HUD::default(),
            fsr: FSR::default(),
            gamemode: false
        }
    }
}

impl From<&Json> for Enhancements {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            hud: value.get("hud")
                .map(HUD::from)
                .unwrap_or(default.hud),

            fsr: value.get("fsr")
                .map(FSR::from)
                .unwrap_or(default.fsr),

            gamemode: value.get("gamemode")
                .and_then(Json::as_bool)
                .unwrap_or(default.gamemode)
        }
    }
}
