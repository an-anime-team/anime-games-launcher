use std::sync::Arc;

use adw::prelude::*;
use relm4::prelude::*;

use mlua::prelude::*;

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::oneshot::Sender as OneshotSender;

use unic_langid::LanguageIdentifier;

use crate::prelude::*;
use crate::ui::components::*;
use crate::ui::windows::download_manager::PipelineActionProgressReport;

pub enum SyncGameCommand {
    /// Get list of available game editions.
    GetEditions(OneshotSender<Result<Vec<GameEdition>, LuaError>>),
    // GetComponents(OneshotSender<Vec<GameComponent<'lua>>>),

    /// Get status of the game installation.
    GetStatus {
        edition: String,
        listener: OneshotSender<Result<InstallationStatus, LuaError>>
    },

    /// Get information about the game launching.
    GetLaunchInfo {
        edition: String,
        listener: OneshotSender<Result<GameLaunchInfo, LuaError>>
    },

    /// Start game diff pipeline execution. This can be
    /// update downloading or full game installation.
    StartDiffPipeline {
        edition: String
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum LibraryPageInput {
    SetGeneration(GenerationManifest),

    AddGameFromGeneration {
        url: String,
        manifest: GameManifest,
        listener: UnboundedSender<SyncGameCommand>
    },

    Activate,

    GameRowSelected(usize),
    HideOtherGamesEditions(DynamicIndex),

    ShowGameDetails {
        game: DynamicIndex,
        variant: Option<DynamicIndex>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryPageOutput {
    SetShowBack(bool)
}

pub struct LibraryPage {
    cards_list: AsyncFactoryVecDeque<CardsList>,
    game_details: AsyncController<GameLibraryDetails>,

    download_manager: AsyncController<DownloadManagerWindow>,

    #[allow(clippy::type_complexity)]
    games: Vec<(String, Arc<GameManifest>, Vec<GameEdition>, UnboundedSender<SyncGameCommand>)>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for LibraryPage {
    type Init = ();
    type Input = LibraryPageInput;
    type Output = LibraryPageOutput;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            adw::NavigationSplitView {
                set_vexpand: true,
                set_hexpand: true,

                #[wrap(Some)]
                set_sidebar = &adw::NavigationPage {
                    // Supress Adwaita-WARNING **: AdwNavigationPage is missing a title
                    set_title: "Games",

                    #[wrap(Some)]
                    set_child = model.cards_list.widget() {
                        add_css_class: "navigation-sidebar",

                        connect_row_activated[sender] => move |_, row| {
                            sender.input(LibraryPageInput::GameRowSelected(row.index() as usize));
                        }
                    }
                },

                #[wrap(Some)]
                set_content = &adw::NavigationPage {
                    set_hexpand: true,

                    // Supress Adwaita-WARNING **: AdwNavigationPage is missing a title
                    set_title: "Details",

                    #[wrap(Some)]
                    set_child = model.game_details.widget(),
                }
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            cards_list: AsyncFactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    CardsListOutput::Selected { card: game, variant }
                        => LibraryPageInput::ShowGameDetails { game, variant },

                    CardsListOutput::HideOtherVariants(index)
                        => LibraryPageInput::HideOtherGamesEditions(index)
                }),

            game_details: GameLibraryDetails::builder()
                .launch(())
                .detach(),

            download_manager: DownloadManagerWindow::builder()
                .launch(())
                .detach(),

            games: Vec::new()
        };

        model.cards_list.widget().connect_row_selected(|_, row| {
            if let Some(row) = row {
                row.emit_activate();
            }
        });

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            LibraryPageInput::SetGeneration(generation) => {
                let config = config::get();

                let packages_store = PackagesStore::new(config.packages.resources_store.path);

                self.games.clear();
                self.cards_list.guard().clear();

                let download_manager_sender = self.download_manager.sender().clone();

                std::thread::spawn(move || {
                    let lua = Lua::new();

                    lua.enable_jit(true);

                    // Iterate through locked resources and find manifests
                    // for appropriate games packages.
                    let mut games_resources = Vec::with_capacity(generation.games.len());

                    for game in generation.games {
                        let resource = generation.lock_file.resources.iter()
                            .find(|resource| game.manifest.package.url == resource.url);

                        if let Some(resource) = resource {
                            games_resources.push((game, resource.clone()));
                        }
                    }

                    // Load generation's lock file into the packages engine.
                    let engine = match PackagesEngine::create(&lua, &packages_store, generation.lock_file) {
                        Ok(engine) => engine,
                        Err(err) => {
                            tracing::error!(?err, "Failed to load locked packages to the lua engine");

                            return;
                        }
                    };

                    // Prepare games engines for locked games.
                    let mut games = Vec::with_capacity(games_resources.len());

                    for (game, resource) in games_resources {
                        tracing::trace!(
                            game = game.manifest.game.title.default_translation(),
                            manifest = resource.url,
                            "Trying to load the game engine"
                        );

                        let Some(integration_resource) = resource.outputs.and_then(|outputs| outputs.get(&game.manifest.package.output).copied()) else {
                            tracing::error!(
                                game = game.manifest.game.title.default_translation(),
                                manifest = resource.url,
                                output = game.manifest.package.output,
                                "Game package doesn't have requested output"
                            );

                            continue;
                        };

                        let module = match engine.load_resource(integration_resource) {
                            Ok(Some(module)) => match module.get::<_, LuaTable>("value") {
                                Ok(module) => module,
                                Err(err) => {
                                    tracing::error!(
                                        game = game.manifest.game.title.default_translation(),
                                        manifest = resource.url,
                                        ?integration_resource,
                                        ?err,
                                        "Failed to get lua table of the game integration"
                                    );

                                    continue;
                                }
                            }

                            Ok(None) => {
                                tracing::error!(
                                    game = game.manifest.game.title.default_translation(),
                                    manifest = resource.url,
                                    ?integration_resource,
                                    "Failed to load game integration module from the lua engine"
                                );

                                continue;
                            }

                            Err(err) => {
                                tracing::error!(
                                    game = game.manifest.game.title.default_translation(),
                                    manifest = resource.url,
                                    ?integration_resource,
                                    ?err,
                                    "Failed to load game integration module from the lua engine"
                                );

                                continue;
                            }
                        };

                        let engine = match GameEngine::from_lua(&lua, &module) {
                            Ok(engine) => engine,
                            Err(err) => {
                                tracing::error!(?err, "Failed to create game integration engine from the loaded package");

                                continue;
                            }
                        };

                        let (listener, receiver) = tokio::sync::mpsc::unbounded_channel();

                        tracing::debug!(
                            url = game.url,
                            title = game.manifest.game.title.default_translation(),
                            "Loaded game integration engine"
                        );

                        sender.input(LibraryPageInput::AddGameFromGeneration {
                            url: game.url,
                            manifest: game.manifest,
                            listener
                        });

                        games.push((engine, receiver, true));
                    }

                    loop {
                        let mut has_working = false;

                        for (game, listener, working) in &mut games {
                            if *working {
                                match listener.try_recv() {
                                    Ok(command) => {
                                        match command {
                                            SyncGameCommand::GetEditions(listener) => {
                                                let _ = listener.send(game.editions());
                                            }

                                            SyncGameCommand::GetStatus { edition, listener } => {
                                                let _ = listener.send(game.game_status(edition));
                                            }

                                            SyncGameCommand::GetLaunchInfo { edition, listener } => {
                                                let _ = listener.send(game.game_launch_info(edition));
                                            }

                                            // TODO: handle errors
                                            SyncGameCommand::StartDiffPipeline { edition } => {
                                                match game.game_diff(edition) {
                                                    Ok(Some(diff)) => {
                                                        download_manager_sender.emit(DownloadManagerWindowMsg::Show);

                                                        // Iterate over actions of the pipeline.
                                                        for action in diff.pipeline() {
                                                            // Get list of handlers for this action.
                                                            let (sender, listener) = flume::bounded(1);

                                                            download_manager_sender.emit(DownloadManagerWindowMsg::PrepareAction {
                                                                diff_title: diff.title().clone(),
                                                                diff_description: diff.description().cloned(),

                                                                action_title: action.title().clone(),
                                                                action_description: action.description().cloned(),

                                                                handlers_listener: sender
                                                            });

                                                            let handlers = match listener.recv() {
                                                                Ok(handlers) => handlers,
                                                                Err(err) => {
                                                                    tracing::error!(?err, "Failed to get pipeline action handlers");

                                                                    break;
                                                                }
                                                            };

                                                            // Process the hook before action execution.
                                                            let result = action.before(move |progress: ProgressReport| {
                                                                (handlers.before)(PipelineActionProgressReport {
                                                                    progress: if let Ok(Some(progress)) = progress.format() {
                                                                        progress
                                                                    } else {
                                                                        LocalizableString::raw(format!("{:.2}%", progress.fraction() * 100.0))
                                                                    },

                                                                    current: progress.progress_current,
                                                                    total: progress.progress_total,

                                                                    title: progress.title,
                                                                    description: progress.description
                                                                })
                                                            });

                                                            // Check hook execution result, if it's `false` then skip the action.
                                                            match result {
                                                                Ok(Some(true)) | Ok(None) => {
                                                                    // Perform the action.
                                                                    let result = action.perform(move |progress: ProgressReport| {
                                                                        (handlers.perform)(PipelineActionProgressReport {
                                                                            progress: if let Ok(Some(progress)) = progress.format() {
                                                                                progress
                                                                            } else {
                                                                                LocalizableString::raw(format!("{:.2}%", progress.fraction() * 100.0))
                                                                            },

                                                                            current: progress.progress_current,
                                                                            total: progress.progress_total,

                                                                            title: progress.title,
                                                                            description: progress.description
                                                                        })
                                                                    });

                                                                    // Check the result of the action execution.
                                                                    if let Err(err) = result {
                                                                        tracing::error!(
                                                                            title = action.title().default_translation(),
                                                                            ?err,
                                                                            "Failed to execute action"
                                                                        );

                                                                        break;
                                                                    }

                                                                    // Process the hook after action execution.
                                                                    let result = action.after(move |progress: ProgressReport| {
                                                                        (handlers.after)(PipelineActionProgressReport {
                                                                            progress: if let Ok(Some(progress)) = progress.format() {
                                                                                progress
                                                                            } else {
                                                                                LocalizableString::raw(format!("{:.2}%", progress.fraction() * 100.0))
                                                                            },

                                                                            current: progress.progress_current,
                                                                            total: progress.progress_total,

                                                                            title: progress.title,
                                                                            description: progress.description
                                                                        })
                                                                    });

                                                                    // Check hook execution result, if it's `false` then skip all the following actions.
                                                                    match result {
                                                                        Ok(Some(true)) | Ok(None) => continue,

                                                                        Ok(Some(false)) => {
                                                                            tracing::debug!(
                                                                                title = action.title().default_translation(),
                                                                                "Diff pipeline skipped"
                                                                            );

                                                                            break;
                                                                        }

                                                                        Err(err) => {
                                                                            tracing::error!(
                                                                                title = action.title().default_translation(),
                                                                                ?err,
                                                                                "Failed to execute action's after hook"
                                                                            );

                                                                            break;
                                                                        }
                                                                    }
                                                                }

                                                                Ok(Some(false)) => {
                                                                    tracing::debug!(
                                                                        title = action.title().default_translation(),
                                                                        "Diff pipeline action skipped"
                                                                    );

                                                                    continue;
                                                                }

                                                                Err(err) => {
                                                                    tracing::error!(
                                                                        title = action.title().default_translation(),
                                                                        ?err,
                                                                        "Failed to execute action's before hook"
                                                                    );

                                                                    break;
                                                                }
                                                            }
                                                        }

                                                        download_manager_sender.emit(DownloadManagerWindowMsg::Hide);
                                                    }

                                                    Ok(None) => tracing::info!("Game diff is not available"),
                                                    Err(err) => tracing::error!(?err, "Failed to get game diff")
                                                }
                                            }
                                        }
                                    }

                                    Err(TryRecvError::Empty) => (),
                                    Err(TryRecvError::Disconnected) => *working = false
                                }

                                has_working |= *working;
                            }
                        }

                        if !has_working {
                            break;
                        }
                    }
                });
            }

            LibraryPageInput::AddGameFromGeneration { url, manifest, listener } => {
                let config = config::get();

                let lang = config.general.language.parse::<LanguageIdentifier>();

                let (send, recv) = tokio::sync::oneshot::channel();

                // TODO: better errors handling
                if let Err(err) = listener.send(SyncGameCommand::GetEditions(send)) {
                    tracing::error!(?err, "Failed to request game's editions");

                    return;
                }

                // TODO: build Arc-s here
                let editions = match recv.await {
                    Ok(Ok(editions)) => editions,

                    Ok(Err(err)) => {
                        tracing::error!(?err, "Failed to request game's editions");

                        return;
                    }

                    Err(err) => {
                        tracing::error!(?err, "Failed to request game's editions");

                        return;
                    }
                };

                self.cards_list.guard().push_back(CardsListInit {
                    image: ImagePath::LazyLoad(manifest.game.images.poster.clone()),

                    title: match &lang {
                        Ok(lang) => manifest.game.title.translate(lang).to_string(),
                        Err(_) => manifest.game.title.default_translation().to_string()
                    },

                    variants: Some(editions.iter()
                        .map(|edition| {
                            match &lang {
                                Ok(lang) => edition.title.translate(lang).to_string(),
                                Err(_) => edition.title.default_translation().to_string()
                            }
                        })
                        .collect::<Vec<_>>())
                });

                self.games.push((url, Arc::new(manifest), editions, listener));
            }

            LibraryPageInput::GameRowSelected(index) => {
                self.cards_list.send(index, CardsListInput::EmitClick);
            }

            LibraryPageInput::HideOtherGamesEditions(index) => {
                self.cards_list.broadcast(CardsListInput::HideVariantsExcept(index));
            }

            LibraryPageInput::ShowGameDetails { game, variant } => {
                // FIXME: don't update details page if it's already open for the given game.

                self.cards_list.broadcast(CardsListInput::HideVariantsExcept(game.clone()));

                // TODO: proper errors handling
                let Some((_, manifest, editions, listener)) = self.games.get(game.current_index()) else {
                    tracing::error!(
                        game = game.current_index(),
                        variant = variant.map(|variant| variant.current_index()),
                        "Failed to read game info"
                    );

                    return;
                };

                let edition = match &variant {
                    Some(variant) => editions.get(variant.current_index()),
                    None => editions.first()
                };

                let Some(edition) = edition.cloned() else {
                    tracing::error!(
                        game = game.current_index(),
                        variant = variant.map(|variant| variant.current_index()),
                        "Failed to get game edition"
                    );

                    return;
                };

                self.game_details.emit(GameLibraryDetailsMsg::SetGameInfo {
                    manifest: manifest.to_owned(),
                    edition,
                    listener: listener.clone()
                });
            }

            LibraryPageInput::Activate => {
                // Update back button visibility when switching pages
            }
        }
    }
}
