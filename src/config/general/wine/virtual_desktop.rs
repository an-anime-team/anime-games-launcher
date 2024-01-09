use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VirtualDesktop {
    pub width: u64,
    pub height: u64,
    pub enabled: bool
}

impl Default for VirtualDesktop {
    #[inline]
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            enabled: false
        }
    }
}

impl From<&Json> for VirtualDesktop {
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            width: value.get("width")
                .and_then(Json::as_u64)
                .unwrap_or(default.width),

            height: value.get("height")
                .and_then(Json::as_u64)
                .unwrap_or(default.height),

            enabled: value.get("enabled")
                .and_then(Json::as_bool)
                .unwrap_or(default.enabled)
        }
    }
}

impl VirtualDesktop {
    #[inline]
    /// `explorer /desktop=[desktop_name],[width]x[height]`
    pub fn get_command(&self, desktop_name: impl AsRef<str>) -> Option<String> {
        if self.enabled {
            Some(format!("explorer /desktop={},{}x{}", desktop_name.as_ref(), self.width, self.height))
        }

        else {
            None
        }
    }
}
