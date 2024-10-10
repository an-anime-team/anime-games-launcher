use std::collections::HashSet;

use gtk::prelude::*;
use relm4::prelude::*;

use serde_json::Value as Json;

use crate::prelude::*;

#[derive(Debug)]
pub enum LoadingWindowMsg {
    SetAction(&'static str),
    LoadMainWindow(GenerationManifest)
}

#[derive(Debug)]
pub struct LoadingWindow {
    main_window: AsyncController<MainWindow>,

    current_action: Option<&'static str>,
    visible: bool
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for LoadingWindow {
    type Init = ();
    type Input = LoadingWindowMsg;
    type Output = ();

    view! {
        window = adw::Window {
            set_title: Some("Anime Games Launcher"),

            set_size_request: (600, 480),
            set_resizable: false,

            add_css_class?: crate::APP_DEBUG.then_some("devel"),

            #[watch]
            set_visible: model.visible,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    add_css_class: "flat"
                },

                adw::StatusPage {
                    set_vexpand: true,
                    set_hexpand: true,

                    set_icon_name: Some(APP_ID),

                    set_title: "Loading",

                    #[watch]
                    set_description: model.current_action
                }
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            main_window: MainWindow::builder()
                .launch(())
                .detach(),

            current_action: None,
            visible: true
        };

        let widgets = view_output!();

        let main_window_sender = model.main_window.sender().clone();

        // TODO: errors handling
        tokio::spawn(async move {
            // Create default folders.
            tracing::debug!("Creating default folders");

            sender.input(LoadingWindowMsg::SetAction("Creating default folders"));

            tokio::try_join!(
                tokio::fs::create_dir_all(DATA_FOLDER.as_path()),
                tokio::fs::create_dir_all(CONFIG_FOLDER.as_path()),
                tokio::fs::create_dir_all(CACHE_FOLDER.as_path()),

                tokio::fs::create_dir_all(&STARTUP_CONFIG.packages.resources_store.path),
                tokio::fs::create_dir_all(&STARTUP_CONFIG.packages.modules_store.path),
                tokio::fs::create_dir_all(&STARTUP_CONFIG.packages.persist_store.path),
                tokio::fs::create_dir_all(&STARTUP_CONFIG.generations.store.path)
            )?;

            // Update the config file to create it
            // if it didn't exist before.
            config::update(&STARTUP_CONFIG)?;

            // Start fetching games manifests for the store page.
            tokio::spawn(async move {
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

                            main_window_sender.emit(MainWindowMsg::AddGamesRegistry { url, manifest });
                        }

                        Err(err) => tracing::error!(?url, ?err, "Failed to await fetching game registry"),
                        Ok(Err(err)) => tracing::error!(?url, ?err, "Failed to fetch game registry")
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

                            main_window_sender.emit(MainWindowMsg::AddGame { url, manifest });
                        }

                        Err(err) => tracing::error!(?url, ?err, "Failed to await fetching game manifest"),
                        Ok(Err(err)) => tracing::error!(?url, ?err, "Failed to fetch game manifest")
                    }
                }

                Ok::<_, anyhow::Error>(())
            });

            // Open generations and packages stores.
            tracing::debug!(
                generations_store = ?STARTUP_CONFIG.generations.store.path,
                packages_store = ?STARTUP_CONFIG.packages.resources_store.path,
                "Opening generations and packages stores"
            );

            sender.input(LoadingWindowMsg::SetAction("Opening generations and packages stores"));

            let generations_store = GenerationsStore::new(&STARTUP_CONFIG.generations.store.path);
            let packages_store = PackagesStore::new(&STARTUP_CONFIG.packages.resources_store.path);

            // List all available generations.
            tracing::debug!("Listing available generations");

            sender.input(LoadingWindowMsg::SetAction("Listing available generations"));

            let mut generations = generations_store.list()?.unwrap_or_default();

            let mut games = None;
            let mut valid_generation = None;

            // Iterate over available generations, from newest to oldest,
            // and try to load them.
            while let Some(generation) = generations.pop() {
                tracing::debug!("Trying to load generation {}", generation.to_base32());

                sender.input(LoadingWindowMsg::SetAction("Loading generation"));

                // Try to load the generation file.
                let Some(generation) = generations_store.load(&generation)? else {
                    tracing::warn!("Generation is missing");

                    continue;
                };

                // Save the added games.
                if games.is_none() {
                    games = Some(generation.games.clone());
                }

                // Validate the generation.
                tracing::debug!("Validating generation resources");

                sender.input(LoadingWindowMsg::SetAction("Validating generation"));

                if !packages_store.validate(&generation.lock_file)? {
                    tracing::warn!("Generation is invalid");

                    continue;
                }

                // Store the valid generation for future use.
                valid_generation = Some(generation);

                break;
            }

            // Start building the new generation with potentially updated games info.
            let new_generation_task = tokio::spawn(async move {
                tracing::debug!("Building the new generation");

                let generation = match games {
                    Some(games) => Generation::with_games(games.into_iter().map(|game| game.url)),
                    None => Generation::new()
                };

                let generation = generation.build(&packages_store, &generations_store).await
                    .map_err(|err| anyhow::anyhow!(err.to_string()))?;

                tracing::debug!("Indexing new generation in the store");

                generations_store.insert(&generation)?;

                Ok::<_, anyhow::Error>(generation)
            });

            // Resolve the generation.
            let valid_generation = match valid_generation {
                Some(generation) => generation,

                // Make a new generation if no valid one was found.
                None => {
                    tracing::debug!("No valid generation found, awaiting the new one");

                    sender.input(LoadingWindowMsg::SetAction("Building new generation"));

                    new_generation_task.await??
                }
            };

            // Load the main window.
            tracing::debug!("Load main window");

            sender.input(LoadingWindowMsg::SetAction("Almost done"));
            sender.input(LoadingWindowMsg::LoadMainWindow(valid_generation));

            Ok::<_, anyhow::Error>(())
        });

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, message: Self::Input, _sender: AsyncComponentSender<Self>) {
        match message {
            LoadingWindowMsg::SetAction(action) => self.current_action = Some(action),

            LoadingWindowMsg::LoadMainWindow(generation) => {
                self.main_window.emit(MainWindowMsg::SetGeneration(generation));
                self.main_window.emit(MainWindowMsg::OpenWindow);

                self.visible = false;
            }
        }
    }
}
