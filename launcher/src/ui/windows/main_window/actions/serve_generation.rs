use adw::prelude::MessageDialogExt;
use gtk::prelude::GtkWindowExt;
use relm4::prelude::*;
use mlua::prelude::*;

use relm4::Sender;

use tokio::sync::oneshot::Sender as OneshotSender;
use tokio::sync::mpsc::error::TryRecvError;

use unic_langid::LanguageIdentifier;

use crate::prelude::*;

#[derive(Debug)]
pub enum SyncGameCommand {
    /// Get list of available game editions.
    GetEditions {
        listener: OneshotSender<Result<Option<Vec<GameEdition>>, LuaError>>
    },

    /// Get status of the game installation.
    GetStatus {
        variant: GameVariant,
        listener: OneshotSender<Result<InstallationStatus, LuaError>>
    },

    /// Get information about the game launching.
    GetLaunchInfo {
        variant: GameVariant,
        listener: OneshotSender<Result<GameLaunchInfo, AsLuaError>>
    },

    /// Start game diff pipeline execution. This can be update downloading
    /// or full game installation. Call provided listener when pipeline
    /// is finished.
    StartDiffPipeline {
        variant: GameVariant,
        listener: OneshotSender<()>
    },

    /// Call `set_property` method with a boolean value.
    SetBoolProperty {
        name: String,
        value: bool
    },

    /// Call `set_property` method with a string value.
    SetStringProperty {
        name: String,
        value: String
    },

    /// Get game settings layout.
    GetSettingsLayout {
        variant: GameVariant,
        listener: OneshotSender<Result<Option<Vec<GameSettingsGroup>>, AsLuaError>>
    }
}

pub fn serve_generation(
    library_page_sender: AsyncComponentSender<LibraryPage>,
    download_manager_sender: Sender<DownloadManagerWindowMsg>,
    generation: GenerationManifest,
    validator: AuthorityValidator,
    local_validator: LocalValidator
) -> anyhow::Result<()> {
    let config = config::get();

    let packages_store = PackagesStore::new(config.packages.resources_store.path);

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
    let options = PackagesEngineOptions {
        show_toast: {
            let library_page_sender = library_page_sender.clone();

            Box::new(move |toast| {
                let config = config::get();

                let language = config.general.language.parse::<LanguageIdentifier>();

                match toast {
                    packages_v1::ToastOptions::Simple(message) => {
                        let message = match &language {
                            Ok(language) => message.translate(language),
                            Err(_) => message.default_translation()
                        }.to_string();

                        library_page_sender.input(LibraryPageInput::Call(Box::new(move |model| {
                            if let Some(toast_overlay) = model.toast_overlay.as_ref() {
                                toast_overlay.add_toast(adw::Toast::new(&message));
                            }
                        })));
                    }

                    packages_v1::ToastOptions::Activatable { message, label, callback } => {
                        let message = match &language {
                            Ok(language) => message.translate(language),
                            Err(_) => message.default_translation()
                        }.to_string();

                        let label = match &language {
                            Ok(language) => label.translate(language),
                            Err(_) => label.default_translation()
                        }.to_string();

                        library_page_sender.input(LibraryPageInput::Call(Box::new(move |model| {
                            if let Some(toast_overlay) = model.toast_overlay.as_ref() {
                                let toast = adw::Toast::new(&message);

                                toast.set_button_label(Some(&label));

                                toast.connect_button_clicked(move |_| {
                                    if let Err(err) = callback.call::<()>(()) {
                                        tracing::error!(?err, "Failed to execute lua engine callback to the toast button");
                                    }
                                });

                                toast_overlay.add_toast(toast);
                            }
                        })));
                    }
                }
            })
        },

        show_notification: Box::new(|options| {
            let config = config::get();

            let language = config.general.language.parse::<LanguageIdentifier>();

            let title = match &language {
                Ok(language) => options.title.translate(language),
                Err(_) => options.title.default_translation()
            };

            let mut notification = notify_rust::Notification::new();

            let mut notification = notification.summary(title)
                .icon(APP_ID)
                .appname(APP_ID);

            if let Some(message) = &options.message {
                let message = match &language {
                    Ok(language) => message.translate(language),
                    Err(_) => message.default_translation()
                };

                notification = notification.body(message);
            }

            if let Some(icon) = &options.icon {
                notification = notification.icon(icon);
            }

            if let Err(err) = notification.show() {
                tracing::error!(?err, "Failed to show system notification");
            }
        }),

        show_dialog: {
            let library_page_sender = library_page_sender.clone();

            Box::new(move |options| {
                let config = config::get();

                let language = config.general.language.parse::<LanguageIdentifier>();

                let title = match &language {
                    Ok(language) => options.title.translate(language),
                    Err(_) => options.title.default_translation()
                }.to_string();

                let message = match &language {
                    Ok(language) => options.message.translate(language),
                    Err(_) => options.message.default_translation()
                }.to_string();

                let (send, recv) = flume::bounded(1);

                library_page_sender.input(LibraryPageInput::Call(Box::new(move |model| {
                    if let Some(window) = model.main_window.as_ref() {
                        let dialog = adw::MessageDialog::new(Some(window), Some(&title), Some(&message));

                        for button in options.buttons {
                            let label = match &language {
                                Ok(language) => button.label.translate(language),
                                Err(_) => button.label.default_translation()
                            };

                            dialog.add_response(&button.name, label);

                            let appearance = match button.status {
                                packages_v1::DialogButtonStatus::Normal    => adw::ResponseAppearance::Default,
                                packages_v1::DialogButtonStatus::Suggested => adw::ResponseAppearance::Suggested,
                                packages_v1::DialogButtonStatus::Dangerous => adw::ResponseAppearance::Destructive
                            };

                            dialog.set_response_appearance(&button.name, appearance);
                        }

                        dialog.connect_response(None, move |_, response| {
                            let _ = send.send(response.to_string());
                        });

                        dialog.present();
                    }
                })));

                recv.recv().ok()
            })
        }
    };

    let engine = match PackagesEngine::create(lua.clone(), &packages_store, generation.lock_file, validator, local_validator, options) {
        Ok(engine) => engine,
        Err(err) => {
            tracing::error!(?err, "Failed to load locked packages to the lua engine");

            return Ok(());
        }
    };

    // Prepare games engines for locked games.
    let mut games = Vec::with_capacity(games_resources.len());

    for (game, resource) in games_resources {
        tracing::trace!(
            game = game.manifest.game.title.default_translation(),
            manifest = resource.url,
            hash = resource.lock.hash.to_base32(),
            "Trying to load the game engine"
        );

        let Some(integration_resource) = resource.outputs.and_then(|outputs| outputs.get(&game.manifest.package.output).copied()) else {
            tracing::error!(
                game = game.manifest.game.title.default_translation(),
                manifest = resource.url,
                hash = resource.lock.hash.to_base32(),
                output = game.manifest.package.output,
                "Game package doesn't have requested output"
            );

            continue;
        };

        let module = match engine.load_resource(integration_resource) {
            Ok(Some(module)) => match module.get::<LuaTable>("value") {
                Ok(module) => module,
                Err(err) => {
                    tracing::error!(
                        game = game.manifest.game.title.default_translation(),
                        manifest = resource.url,
                        hash = resource.lock.hash.to_base32(),
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
                    hash = resource.lock.hash.to_base32(),
                    ?integration_resource,
                    "Failed to load game integration module from the lua engine"
                );

                continue;
            }

            Err(err) => {
                tracing::error!(
                    game = game.manifest.game.title.default_translation(),
                    manifest = resource.url,
                    hash = resource.lock.hash.to_base32(),
                    ?integration_resource,
                    ?err,
                    "Failed to load game integration module from the lua engine"
                );

                continue;
            }
        };

        let engine = match GameEngine::from_lua(lua.clone(), &module) {
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

        library_page_sender.input(LibraryPageInput::AddGameFromGeneration {
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
                            SyncGameCommand::GetEditions { listener } => {
                                let _ = listener.send(game.editions(*CURRENT_PLATFORM));
                            }

                            SyncGameCommand::GetStatus { variant, listener } => {
                                let _ = listener.send(game.game_status(&variant));
                            }

                            SyncGameCommand::GetLaunchInfo { variant, listener } => {
                                let info = game.game_launch_info(&variant);

                                let _ = listener.send(info);
                            }

                            // TODO: handle errors
                            SyncGameCommand::StartDiffPipeline { variant, listener } => {
                                match game.game_diff(&variant) {
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

                                let _ = listener.send(());
                            }

                            SyncGameCommand::SetBoolProperty { name, value } => {
                                if let Err(err) = game.set_property(&name, LuaValue::Boolean(value)) {
                                    tracing::error!(?name, ?value, ?err, "Failed to set property value");
                                }
                            }

                            SyncGameCommand::SetStringProperty { name, value } => {
                                match lua.create_string(&value) {
                                    Ok(lua_value) => {
                                        if let Err(err) = game.set_property(&name, LuaValue::String(lua_value)) {
                                            tracing::error!(?name, ?value, ?err, "Failed to set property value");
                                        }
                                    }

                                    Err(err) => tracing::error!(?err, "Failed to cast property value to lua string")
                                }
                            }

                            SyncGameCommand::GetSettingsLayout { variant, listener } => {
                                let _ = listener.send(game.get_settings_layout(&variant));
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

    Ok(())
}
