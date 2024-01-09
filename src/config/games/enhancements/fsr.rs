use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FsrQuality {
    /// `WINE_FULLSCREEN_FSR_MODE=ultra`
    Ultra,

    /// `WINE_FULLSCREEN_FSR_MODE=quality`
    Quality,

    /// `WINE_FULLSCREEN_FSR_MODE=balanced`
    Balanced,

    /// `WINE_FULLSCREEN_FSR_MODE=performance`
    Performance
}

impl Default for FsrQuality {
    #[inline]
    fn default() -> Self {
        Self::Balanced
    }
}

impl From<&Json> for FsrQuality {
    #[inline]
    fn from(value: &Json) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FSR {
    /// Sharpening strength, from 0 to 5 where 0 is maximum and 5 is minimum sharpening
    /// 
    /// Default: 2
    pub strength: u64,
    pub quality: FsrQuality,
    pub enabled: bool
}

impl Default for FSR {
    #[inline]
    fn default() -> Self {
        Self {
            strength: 2,
            quality: FsrQuality::default(),
            enabled: true
        }
    }
}

impl From<&Json> for FSR {
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            strength: match value.get("strength") {
                Some(value) => value.as_u64().unwrap_or(default.strength),
                None => default.strength
            },

            quality: match value.get("quality") {
                Some(value) => FsrQuality::from(value),
                None => default.quality
            },

            enabled: match value.get("enabled") {
                Some(value) => value.as_bool().unwrap_or(default.enabled),
                None => default.enabled
            }
        }
    }
}

impl FSR {
    /// Get environment variables corresponding to used amd fsr options
    pub fn get_env_vars(&self) -> HashMap<&str, String> {
        if self.enabled {
            let mut env = HashMap::from([
                ("WINE_FULLSCREEN_FSR", String::from("1")),
                ("WINE_FULLSCREEN_FSR_STRENGTH", self.strength.to_string())
            ]);

            // Set FSR quality mode if some is selected
            // https://github.com/GloriousEggroll/wine-ge-custom/releases/tag/GE-Proton7-25
            env.insert("WINE_FULLSCREEN_FSR_MODE", match self.quality {
                FsrQuality::Ultra       => String::from("ultra"),
                FsrQuality::Quality     => String::from("quality"),
                FsrQuality::Balanced    => String::from("balanced"),
                FsrQuality::Performance => String::from("performance")
            });

            env
        }

        else {
            // FSR is enabled by default, so if it's disabled in the launcher
            // I should use this variable to really disable it
            HashMap::from([
                ("WINE_FULLSCREEN_FSR", String::from("0"))
            ])
        }
    }
}
