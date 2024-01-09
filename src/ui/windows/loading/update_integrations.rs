use std::collections::HashMap;

use serde_json::Value as Json;

use anime_game_core::network::minreq;

use crate::config;
use crate::games::integrations::manifest::Manifest;

// TODO: parallelize this

#[inline]
pub fn update_integrations() -> anyhow::Result<()> {
    let config = config::get();

    let mut games = HashMap::new();

    for source in config.games.integrations.sources {
        let integrations = minreq::get(format!("{source}/integrations.json"))
            .send()?.json::<Json>()?;

        let Some(integrations) = integrations.get("games").and_then(Json::as_array) else {
            anyhow::bail!("Wrong integrations file structue");
        };

        for game in integrations {
            if let Some(game) = game.as_str() {
                let bytes = minreq::get(format!("{source}/games/{game}/manifest.json"))
                    .send()?.into_bytes();

                let manifest = Manifest::from_json(&serde_json::from_slice(&bytes)?)?;

                games.insert(game.to_string(), (source.clone(), manifest, bytes));
            }
        }
    }

    for (game, (source, manifest, bytes)) in games {
        let integration_path = config.games.integrations.path.join(&game);

        let manifest_path = integration_path.join("manifest.json");
        let script_path = integration_path.join(&manifest.script_path);

        if integration_path.exists() {
            let local_manifest = std::fs::read(&manifest_path)?;
            let local_manifest = serde_json::from_slice(&local_manifest)?;
            let local_manifest = Manifest::from_json(&local_manifest)?;

            if local_manifest.script_version == manifest.script_version {
                continue;
            }
        }

        else {
            std::fs::create_dir_all(&integration_path)?;
        }

        let script = minreq::get(format!("{source}/games/{game}/{}", &manifest.script_path))
            .send()?.into_bytes();

        std::fs::write(manifest_path, bytes)?;
        std::fs::write(script_path, script)?;
    }

    Ok(())
}
