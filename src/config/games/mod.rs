use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

use crate::LAUNCHER_FOLDER;

use crate::config::driver::Driver;

pub mod genshin;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Games {
    pub integrations: Driver,
    pub genshin: genshin::Genshin
}

impl Default for Games {
    #[inline]
    fn default() -> Self {
        Self {
            integrations: Driver::PhysicalFsDriver {
                base_folder: LAUNCHER_FOLDER.join("integrations")
            },

            genshin: genshin::Genshin::default()
        }
    }
}

impl From<&Json> for Games {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            integrations: value.get("integrations")
                .and_then(|value| serde_json::from_value(value.clone()).ok())
                .unwrap_or(default.integrations),

            genshin: value.get("genshin")
                .map(genshin::Genshin::from)
                .unwrap_or(default.genshin),
        }
    }
}
