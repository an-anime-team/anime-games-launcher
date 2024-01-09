use serde_json::Value as Json;

use super::standards::IntegrationStandard;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Manifest {
    pub game_name: String,
    pub game_title: String,
    pub game_developer: String,

    pub script_path: String,
    pub script_version: String,
    pub script_standard: IntegrationStandard
}

impl Manifest {
    pub fn from_json(manifest: &Json) -> anyhow::Result<Self> {
        match manifest.get("manifest_version").and_then(Json::as_str) {
            Some("1") => {
                let Some(game_manifest) = manifest.get("game") else {
                    anyhow::bail!("Wrong manifest v1 structure: field `game` expected but wasn't presented");
                };

                let Some(script_manifest) = manifest.get("script") else {
                    anyhow::bail!("Wrong manifest v1 structure: field `script` expected but wasn't presented");
                };

                Ok(Self {
                    game_name: game_manifest.get("name")
                        .and_then(Json::as_str)
                        .ok_or_else(|| anyhow::anyhow!("Wrong manifest v1 structure: field `game.name` expected but wasn't presented"))?
                        .to_string(),

                    game_title: game_manifest.get("title")
                        .and_then(Json::as_str)
                        .ok_or_else(|| anyhow::anyhow!("Wrong manifest v1 structure: field `game.title` expected but wasn't presented"))?
                        .to_string(),

                    game_developer: game_manifest.get("developer")
                        .and_then(Json::as_str)
                        .ok_or_else(|| anyhow::anyhow!("Wrong manifest v1 structure: field `game.developer` expected but wasn't presented"))?
                        .to_string(),

                    script_path: script_manifest.get("path")
                        .and_then(Json::as_str)
                        .ok_or_else(|| anyhow::anyhow!("Wrong manifest v1 structure: field `script.path` expected but wasn't presented"))?
                        .to_string(),

                    script_version: script_manifest.get("version")
                        .and_then(Json::as_str)
                        .ok_or_else(|| anyhow::anyhow!("Wrong manifest v1 structure: field `script.version` expected but wasn't presented"))?
                        .to_string(),

                    script_standard: match script_manifest.get("standard").and_then(Json::as_str) {
                        Some("1") => IntegrationStandard::V1,

                        Some(version) => anyhow::bail!("Wrong manifest v1 structure: field `script.standard` containts unknown version: {version}"),
                        None => anyhow::bail!("Wrong manifest v1 structure: field `script.standard` expected but wasn't presented")
                    }
                })
            }

            Some(version) => anyhow::bail!("Unknown manifest version: {version}"),
            None => anyhow::bail!("Wrong manifest file structure")
        }
    }
}
