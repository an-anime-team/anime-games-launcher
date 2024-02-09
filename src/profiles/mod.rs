use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use crate::config;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Profile {
    pub name: String
}

impl Profile {
    pub fn path(&self) -> PathBuf {
        config::get().components.wine.prefix.path.join(&self.name)
    }
}
