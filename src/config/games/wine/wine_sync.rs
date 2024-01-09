use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WineSync {
    None,
    ESync,
    FSync
}

impl Default for WineSync {
    #[inline]
    fn default() -> Self {
        Self::FSync
    }
}

impl From<&Json> for WineSync {
    #[inline]
    fn from(value: &Json) -> Self {
        serde_json::from_value(value.clone()).unwrap_or_default()
    }
}

impl WineSync {
    /// Get environment variables corresponding to used wine sync
    pub fn get_env_vars(&self) -> HashMap<&str, &str> {
        let key = match self {
            Self::None => return HashMap::new(),

            Self::ESync => "WINEESYNC",
            Self::FSync => "WINEFSYNC"
        };

        HashMap::from([(key, "1")])
    }
}
