use std::path::{Path, PathBuf};

use serde_json::Value as Json;

pub mod manifest;
pub mod standards;
pub mod driver;

use manifest::Manifest;
use driver::Driver;

#[derive(Debug)]
pub struct Game {
    pub manifest: Manifest,
    pub driver: Driver
}

impl Game {
    pub fn new(manifest_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let manifest = std::fs::read(manifest_path.as_ref())?;
        let manifest = serde_json::from_slice::<Json>(&manifest)?;
        let manifest = Manifest::from_json(&manifest)?;

        let script_path = PathBuf::from(&manifest.script_path);

        let script_path = if script_path.is_absolute() {
            script_path
        } else {
            manifest_path.as_ref()
                .parent()
                .map(|path| path.join(&script_path))
                .unwrap_or(script_path)
        };

        let driver = Driver::new(
            &manifest.game_name,
            manifest.script_standard,
            std::fs::read_to_string(script_path)?
        )?;

        Ok(Self {
            manifest,
            driver
        })
    }
}
