use serde::{Serialize, Deserialize};

use serde_json::Value as Json;

pub mod paths;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Genshin {
    pub paths: paths::Paths
}

impl From<&Json> for Genshin {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            paths: value.get("paths")
                .map(paths::Paths::from)
                .unwrap_or(default.paths),
        }
    }
}
