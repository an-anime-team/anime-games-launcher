use std::collections::HashMap;

use serde_json::Value as Json;

use crate::packages::prelude::*;

/// Parse manifest v1 file from given JSON object
pub async fn parse_v1(manifest: &Json, uri: String) -> anyhow::Result<Manifest> {
    let Some(game) = manifest.get("game") else {
        anyhow::bail!("Incorrect manifest v1 file format: `game` field is missing")
    };

    let Some(game_name) = game.get("name").and_then(Json::as_str) else {
        anyhow::bail!("Incorrect manifest v1 file format: `game.name` field is missing")
    };

    let Some(game_title) = game.get("title").and_then(Json::as_str) else {
        anyhow::bail!("Incorrect manifest v1 file format: `game.title` field is missing")
    };

    let Some(script) = manifest.get("script") else {
        anyhow::bail!("Incorrect manifest v1 file format: `script` field is missing")
    };

    let Some(script_path) = script.get("path").and_then(Json::as_str) else {
        anyhow::bail!("Incorrect manifest v1 file format: `script.path` field is missing")
    };

    let Some(script_standard) = script.get("standard").and_then(Json::as_str) else {
        anyhow::bail!("Incorrect manifest v1 file format: `script.standard` field is missing")
    };

    let script_body = crate::handlers::handle(format!("{uri}/{script_path}"))?
        .join().await?
        .map_err(|err| anyhow::anyhow!("Failed to request package's integration script: {err}"))?;

    Ok(Manifest {
        manifest_version: 1,
        metadata: ManifestMetadata {
            homepage: None,
            maintainers: None
        },
        inputs: HashMap::new(),
        outputs: vec![
            ManifestOutput {
                format: ManifestOutputFormat::Integration,
                path: script_path.to_string(),
                hash: Hash::from_slice(HashAlgorithm::Xxh3, &script_body),
                metadata: ManifestOutputMetadata {
                    uuid: Uuid::new_from_str(game_name),
                    name: game_name.to_string(),
                    title: game_title.to_string(),
                    standard: script_standard.parse::<u64>()?
                }
            }
        ]
    })
}
