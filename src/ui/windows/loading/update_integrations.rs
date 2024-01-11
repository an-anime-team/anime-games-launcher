use std::collections::HashMap;

use serde_json::Value as Json;

use anime_game_core::network::minreq;

use crate::config;
use crate::games::integrations::manifest::Manifest;

struct IntegrationInfo {
    pub source: String,
    pub manifest_body: Vec<u8>,
    pub manifest: Manifest
}

#[inline]
pub fn update_integrations(pool: &rusty_pool::ThreadPool) -> anyhow::Result<()> {
    let config = config::get();

    let mut tasks = Vec::with_capacity(config.games.integrations.sources.len());

    for source in config.games.integrations.sources {
        tasks.push(pool.evaluate(move || -> anyhow::Result<HashMap<String, IntegrationInfo>> {
            let response = minreq::get(format!("{source}/integrations.json"))
                .send()?;

            let mut games = HashMap::new();

            // HTTP OK
            if (200..300).contains(&response.status_code) {
                let integrations = response.json::<Json>()?;

                let Some(integrations) = integrations.get("games").and_then(Json::as_array) else {
                    anyhow::bail!("Wrong integrations file structue");
                };

                for game in integrations {
                    if let Some(game) = game.as_str() {
                        let bytes = minreq::get(format!("{source}/games/{game}/manifest.json"))
                            .send()?.into_bytes();

                        let manifest = Manifest::from_json(&serde_json::from_slice(&bytes)?)?;

                        games.insert(game.to_string(), IntegrationInfo {
                            source: format!("{source}/games/{game}"),
                            manifest_body: bytes,
                            manifest
                        });
                    }
                }
            }

            else {
                let response = minreq::get(format!("{source}/manifest.json"))
                    .send()?;

                // HTTP OK
                if (200..300).contains(&response.status_code) {
                    let bytes = response.into_bytes();

                    let manifest = Manifest::from_json(&serde_json::from_slice(&bytes)?)?;

                    games.insert(manifest.game_name.to_string(), IntegrationInfo {
                        source: source.clone(),
                        manifest_body: bytes,
                        manifest
                    });
                }

                else {
                    anyhow::bail!("Source {source} doesn't have integrations.json or manifest.json file");
                }
            }

            Ok(games)
        }));
    }

    let mut games = HashMap::new();

    for task in tasks {
        for (game, value) in task.await_complete()? {
            games.insert(game, value);
        }
    }

    let mut tasks = Vec::with_capacity(games.len());

    for (game, info) in games {
        let integration_path = config.games.integrations.path.join(&game);

        let manifest_path = integration_path.join("manifest.json");
        let script_path = integration_path.join(&info.manifest.script_path);

        // Spawning new threads to read a few KBs of data is more time-consuming
        // than doing it in the same thread
        if integration_path.exists() {
            let local_manifest = std::fs::read(&manifest_path)?;
            let local_manifest = serde_json::from_slice(&local_manifest)?;
            let local_manifest = Manifest::from_json(&local_manifest)?;

            if local_manifest.script_version == info.manifest.script_version {
                continue;
            }
        }

        else {
            std::fs::create_dir_all(&integration_path)?;
        }

        tasks.push(pool.evaluate(move || -> anyhow::Result<()> {
            let script = minreq::get(format!("{}/{}", info.source, &info.manifest.script_path))
                .send()?.into_bytes();

            std::fs::write(manifest_path, info.manifest_body)?;
            std::fs::write(script_path, script)?;

            Ok(())
        }));
    }

    tasks.into_iter().try_for_each(|task| task.await_complete())?;

    Ok(())
}
