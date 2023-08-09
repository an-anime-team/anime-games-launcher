use serde::{Serialize, Deserialize};

use serde_json::Value as Json;

pub mod genshin;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Games {
    pub genshin: genshin::Genshin
}

impl From<&Json> for Games {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            genshin: value.get("genshin")
                .map(genshin::Genshin::from)
                .unwrap_or(default.genshin),
        }
    }
}
