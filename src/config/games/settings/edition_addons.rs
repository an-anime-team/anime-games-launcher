use serde::{Serialize, Deserialize};
use serde_json::Value as Json;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GameEditionAddon {
    pub group: String,
    pub name: String
}

impl From<&Json> for GameEditionAddon {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            group: value.get("group")
                .and_then(Json::as_str)
                .map(String::from)
                .unwrap_or(default.group),

            name: value.get("name")
                .and_then(Json::as_str)
                .map(String::from)
                .unwrap_or(default.name)
        }
    }
}
