use serde::{Serialize, Deserialize};

use serde_json::Value as Json;

pub mod general;
pub mod components;
pub mod games;

use crate::CONFIG_FILE;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub general: general::General,
    pub components: components::Components,
    pub games: games::Games
}

impl From<&Json> for Config {
    #[inline]
    fn from(value: &Json) -> Self {
        let default = Self::default();

        Self {
            general: value.get("general")
                .map(general::General::from)
                .unwrap_or(default.general),

            components: value.get("components")
                .map(components::Components::from)
                .unwrap_or(default.components),

            games: value.get("games")
                .map(games::Games::from)
                .unwrap_or(default.games),
        }
    }
}

/// Read configuration from the file
pub fn get() -> Config {
    std::fs::read(CONFIG_FILE.as_path()).ok()
        .and_then(|config| serde_json::from_slice::<Json>(&config).ok())
        .map(|config| Config::from(&config))
        .unwrap_or_default()
}

/// Update configuration file's value
pub fn set(property: impl AsRef<str>, value: impl Into<Json>) -> anyhow::Result<()> {
    let mut config = std::fs::read(CONFIG_FILE.as_path()).ok()
        .and_then(|config| serde_json::from_slice::<Json>(&config).ok())
        .unwrap_or_else(|| serde_json::to_value(Config::default()).unwrap());

    let mut nested_config = &mut config;

    for property in property.as_ref().split('.') {
        nested_config = &mut nested_config[property];
    }

    *nested_config = value.into();

    Ok(std::fs::write(CONFIG_FILE.as_path(), serde_json::to_string_pretty(&config)?)?)
}

/// Update configuration file
pub fn update(config: &Config) -> anyhow::Result<()> {
    Ok(std::fs::write(CONFIG_FILE.as_path(), serde_json::to_string_pretty(config)?)?)
}
