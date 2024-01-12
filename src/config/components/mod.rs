use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::i18n;

pub mod wine;
pub mod dxvk;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Components {
    pub channel: String,
    pub wine: wine::Wine,
    pub dxvk: dxvk::Dxvk
}

impl Default for Components {
    #[inline]
    fn default() -> Self {
        Self {
            channel: if i18n::get_system_language() == "zh_cn" {
                String::from("https://raw.gitmirror.com/an-anime-team/components/main")
            } else {
                String::from("https://raw.githubusercontent.com/an-anime-team/components/main")
            },

            wine: wine::Wine::default(),
            dxvk: dxvk::Dxvk::default()
        }
    }
}

impl From<&Json> for Components {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            channel: value.get("channel")
                .and_then(Json::as_str)
                .map(String::from)
                .unwrap_or(default.channel),

            wine: value.get("wine")
                .map(wine::Wine::from)
                .unwrap_or(default.wine),

            dxvk: value.get("dxvk")
                .map(dxvk::Dxvk::from)
                .unwrap_or(default.dxvk)
        }
    }
}
