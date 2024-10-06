use serde::{Serialize, Deserialize};
use serde_json::{json, Value as Json};

use crate::prelude::*;

pub mod general;
pub mod packages;
pub mod generations;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Config {
    pub general: general::General,
    pub packages: packages::Packages,
    pub generations: generations::Generations
}

impl AsJson for Config {
    fn to_json(&self) -> Result<Json, AsJsonError> {
        Ok(json!({
            "general": self.general.to_json()?,
            "packages": self.packages.to_json()?,
            "generations": self.generations.to_json()?
        }))
    }

    fn from_json(json: &Json) -> Result<Self, AsJsonError> where Self: Sized {
        Ok(Self {
            general: json.get("general")
                .map(general::General::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("general"))??,

            packages: json.get("packages")
                .map(packages::Packages::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("packages"))??,

            generations: json.get("generations")
                .map(generations::Generations::from_json)
                .ok_or_else(|| AsJsonError::FieldNotFound("generations"))??
        })
    }
}

/// Read configuration from the file.
pub fn get() -> Config {
    std::fs::read(CONFIG_FILE.as_path()).ok()
        .and_then(|config| serde_json::from_slice::<Json>(&config).ok())
        .and_then(|config| Config::from_json(&config).ok())
        .unwrap_or_default()
}

/// Change configuration file field's value.
pub fn set(property: impl AsRef<str>, value: impl Into<Json>) -> anyhow::Result<()> {
    let mut config = std::fs::read(CONFIG_FILE.as_path()).ok()
        .and_then(|config| serde_json::from_slice::<Json>(&config).ok())
        .unwrap_or_else(|| serde_json::to_value(Config::default()).unwrap());

    let mut nested_config = &mut config;

    for field in property.as_ref().split('.') {
        nested_config = nested_config.get_mut(field)
            .ok_or_else(|| anyhow::anyhow!("Failed to index '{field}' in '{}'", property.as_ref()))?;
    }

    *nested_config = value.into();

    Ok(std::fs::write(CONFIG_FILE.as_path(), serde_json::to_string_pretty(&config)?)?)
}

/// Update configuration file
pub fn update(config: &Config) -> anyhow::Result<()> {
    Ok(std::fs::write(CONFIG_FILE.as_path(), serde_json::to_string_pretty(config)?)?)
}
