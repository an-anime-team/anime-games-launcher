use std::collections::HashSet;

use serde_json::Value as Json;

use crate::prelude::*;

/// Fetch games registries specified in the config file
/// and send found games to the given callback.
///
/// Return error only in critical situations.
/// Broken links and json files will be logged and skipped.
pub async fn fetch_games(callback: impl Fn(String, GameManifest)) -> anyhow::Result<()> {
    let client = STARTUP_CONFIG.general.network.builder()?.build()?;

    let mut registries_tasks = Vec::with_capacity(STARTUP_CONFIG.games.registries.len());

    // Start fetching the registries.
    tracing::debug!("Fetching games registries");

    for url in STARTUP_CONFIG.games.registries.clone() {
        let request = client.get(&url);

        let task = tokio::spawn(async move {
            let response = request.send().await?
                .bytes().await?;

            let manifest = serde_json::from_slice::<Json>(&response)?;
            let manifest = GamesRegistryManifest::from_json(&manifest)?;

            Ok::<_, anyhow::Error>(manifest)
        });

        registries_tasks.push((url, task));
    }

    // Await registries fetching.
    let mut games = HashSet::new();

    for (url, task) in registries_tasks.drain(..) {
        tracing::trace!(?url, "Awaiting game registry");

        match task.await {
            Ok(Ok(manifest)) => {
                tracing::trace!(
                    ?url,
                    title = manifest.title.default_translation(),
                    "Added game registry"
                );

                for game in &manifest.games {
                    games.insert(game.url.clone());
                }
            }

            Err(err) => tracing::error!(?url, ?err, "Failed to await fetching games registry"),
            Ok(Err(err)) => tracing::error!(?url, ?err, "Failed to fetch games registry")
        }
    }

    // Start fetching games.
    tracing::debug!("Fetching games manifests");

    let mut games_tasks = Vec::with_capacity(games.len());

    for url in games.drain() {
        let request = client.get(&url);

        let task = tokio::spawn(async move {
            let response = request.send().await?
                .bytes().await?;

            let manifest = serde_json::from_slice::<Json>(&response)?;
            let manifest = GameManifest::from_json(&manifest)?;

            Ok::<_, anyhow::Error>(manifest)
        });

        games_tasks.push((url, task));
    }

    // Await games fetching.
    for (url, task) in games_tasks.drain(..) {
        tracing::trace!(?url, "Awaiting game manifest");

        match task.await {
            Ok(Ok(manifest)) => {
                tracing::trace!(
                    ?url,
                    title = manifest.game.title.default_translation(),
                    "Added game manifest"
                );

                callback(url, manifest);
            }

            Err(err) => tracing::error!(?url, ?err, "Failed to await fetching game manifest"),
            Ok(Err(err)) => tracing::error!(?url, ?err, "Failed to fetch game manifest")
        }
    }

    Ok::<_, anyhow::Error>(())
}
