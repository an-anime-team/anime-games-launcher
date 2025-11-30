use serde::{Serialize, Deserialize};
use serde_json::Value as Json;
use std::sync::Mutex;
use std::time::SystemTime;

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

/// Cache structure to hold the config and its file modification time
struct ConfigCache {
    config: Config,
    file_modified_time: SystemTime,
}

lazy_static::lazy_static! {
    static ref CONFIG_CACHE: Mutex<Option<ConfigCache>> = Mutex::new(None);
}

/// Get the file modification time
fn get_file_modified_time() -> Option<SystemTime> {
    std::fs::metadata(CONFIG_FILE.as_path())
        .and_then(|metadata| metadata.modified())
        .ok()
}

/// Read configuration from the file
pub fn get() -> Config {
    // Try to get current file modification time
    let current_modified_time = get_file_modified_time();

    // Lock the cache
    let mut cache = CONFIG_CACHE.lock().unwrap();

    // Check if cache is valid
    if let Some(cached) = cache.as_ref() {
        if current_modified_time == Some(cached.file_modified_time) {
            return cached.config.clone();
        }
    }

    // Cache miss or file was modified, read from disk
    let config = std::fs::read(CONFIG_FILE.as_path()).ok()
        .and_then(|config| serde_json::from_slice::<Json>(&config).ok())
        .map(|config| Config::from(&config))
        .unwrap_or_default();

    // Update cache if we have a valid modification time
    if let Some(modified_time) = current_modified_time {
        *cache = Some(ConfigCache {
            config: config.clone(),
            file_modified_time: modified_time,
        });
    }

    config
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

    // Write to file and invalidate cache
    std::fs::write(CONFIG_FILE.as_path(), serde_json::to_string_pretty(&config)?)?;
    
    // Clear cache to force reload on next get()
    let mut cache = CONFIG_CACHE.lock().unwrap();
    *cache = None;

    Ok(())
}

/// Update configuration file
pub fn update(config: &Config) -> anyhow::Result<()> {
    std::fs::write(CONFIG_FILE.as_path(), serde_json::to_string_pretty(config)?)?;
    
    // Clear cache to force reload on next get()
    let mut cache = CONFIG_CACHE.lock().unwrap();
    *cache = None;

    Ok(())
}
