// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::sync::Arc;

use relm4::prelude::*;
use adw::prelude::*;

use agl_packages::storage::Storage;
use agl_runtime::mlua::prelude::*;
use agl_runtime::runtime::{Runtime, ModulePaths};
use agl_games::engine::{
    GameEdition,
    GameVariant,
    GameIntegration,
    ActionsPipeline,
    GameSettingsGroup
};

use crate::{config, consts};
use crate::games::GameLock;
use crate::ui::dialogs;
use crate::ui::components::lazy_picture::ImagePath;
use crate::ui::components::cards_list::{
    CardsList, CardsListInit, CardsListInput, CardsListOutput
};
use crate::ui::components::game_library_details::{
    GameLibraryDetails, GameLibraryDetailsInput, GameLibraryDetailsOutput
};

#[derive(Debug, Clone)]
struct GameInfo {
    pub lock: GameLock,
    pub integration: Arc<GameIntegration>,
    pub editions: Option<Box<[GameEdition]>>,
    pub card_index: DynamicIndex
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryPageInput {
    AddGame(GameLock),

    SelectGameWithUrl(String),

    SelectGameWithIndex {
        game: usize,
        variant: Option<usize>
    },

    CollapseGamesExceptIndex(DynamicIndex),

    UpdateSelectedGameInfo
}

#[derive(Debug, Clone)]
pub enum LibraryPageOutput {
    ScheduleGameActionsPipeline {
        game_index: usize,
        game_title: String,
        actions_pipeline: Arc<ActionsPipeline>
    },

    OpenGameSettingsWindow {
        variant: GameVariant,
        integration: Arc<GameIntegration>,
        layout: Box<[GameSettingsGroup]>
    }
}

pub struct LibraryPage {
    cards_list: AsyncFactoryVecDeque<CardsList>,
    game_details: AsyncController<GameLibraryDetails>,
    // download_manager: AsyncController<DownloadManagerWindow>,

    // main_window: Option<adw::ApplicationWindow>,
    // toast_overlay: Option<adw::ToastOverlay>,

    storage: Storage,
    runtime: Runtime,

    games: Vec<GameInfo>
}

impl std::fmt::Debug for LibraryPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibraryPage")
            .field("cards_list", &self.cards_list)
            .field("storage", &self.storage)
            .field("runtime", &"Runtime")
            .field("games", &self.games)
            .finish()
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for LibraryPage {
    type Init = ();
    type Input = LibraryPageInput;
    type Output = LibraryPageOutput;

    view! {
        #[root]
        adw::ToastOverlay {
            adw::NavigationSplitView {
                set_vexpand: true,
                set_hexpand: true,

                #[wrap(Some)]
                set_sidebar = &adw::NavigationPage {
                    set_title: "Games",

                    #[wrap(Some)]
                    set_child = model.cards_list.widget() {
                        add_css_class: "navigation-sidebar",

                        connect_row_activated[sender] => move |_, row| {
                            sender.input(LibraryPageInput::SelectGameWithIndex {
                                game: row.index() as usize,
                                variant: None
                            });
                        }
                    }
                },

                #[wrap(Some)]
                set_content = &adw::NavigationPage {
                    set_title: "Details",

                    set_hexpand: true,

                    #[wrap(Some)]
                    set_child = model.game_details.widget(),
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            cards_list: AsyncFactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    CardsListOutput::Selected { card, variant } => {
                        LibraryPageInput::SelectGameWithIndex {
                            game: card.current_index(),
                            variant: variant.map(|variant| variant.current_index())
                        }
                    }

                    CardsListOutput::HideOtherVariants(index)
                        => LibraryPageInput::CollapseGamesExceptIndex(index)
                }),

            game_details: GameLibraryDetails::builder()
                .launch(())
                .forward(sender.output_sender(), |msg| match msg {
                    GameLibraryDetailsOutput::ScheduleGameActionsPipeline { game_index, game_title, actions_pipeline }
                        => LibraryPageOutput::ScheduleGameActionsPipeline { game_index, game_title, actions_pipeline },

                    GameLibraryDetailsOutput::OpenGameSettingsWindow { variant, integration, layout }
                        => LibraryPageOutput::OpenGameSettingsWindow { variant, integration, layout }
                }),

            // download_manager: DownloadManagerWindow::builder()
            //     .launch(())
            //     .detach(),

            // main_window: None,
            // toast_overlay: None,

            storage: Storage::open(&config::startup().packages_resources_path)
                .expect("failed to open packages storage"),

            runtime: Runtime::new()
                .expect("failed to initialize packages runtime"),

            games: Vec::new()
        };

        model.cards_list.widget().connect_row_selected(|_, row| {
            if let Some(row) = row {
                row.emit_activate();
            }
        });

        let widgets = view_output!();

        // model.main_window = Some(parent);
        // model.toast_overlay = Some(widgets.toast_overlay.clone());

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>
    ) {
        match msg {
            LibraryPageInput::AddGame(game_lock) => {
                tracing::debug!(
                    url = game_lock.url,
                    title = game_lock.manifest.game.title.default_translation(),
                    "loading game package"
                );

                let config = config::get();
                let lang = config.language();

                let title = match &lang {
                    Ok(lang) => game_lock.manifest.game.title.translate(lang),
                    Err(_) => game_lock.manifest.game.title.default_translation()
                };

                let paths = ModulePaths {
                    temp_folder: config.packages_temporary_path.clone(),
                    modules_folder: config.packages_modules_path.clone(),
                    persistent_folder: config.packages_persistent_path.clone()
                };

                if let Err(err) = self.runtime.load_packages(&game_lock.lock, &self.storage, &paths) {
                    tracing::error!(
                        ?err,
                        url = game_lock.url,
                        title = game_lock.manifest.game.title.default_translation(),
                        "failed to load game package"
                    );

                    dialogs::error(
                        format!("Failed to load {title} game package"),
                        err.to_string()
                    );

                    return;
                }

                fn find_module_key(lock: &GameLock) -> Option<String> {
                    for hash in &lock.lock.root {
                        #[allow(clippy::collapsible_if)]
                        if let Some(package) = lock.lock.packages.get(hash) {
                            if let Some(output) = package.outputs.get(&lock.manifest.package.output) {
                                // TODO: can change in future. Better make some
                                //       universal solution.
                                let module_key = format!("{}#module", output.hash.to_base32());

                                return Some(module_key);
                            }
                        }
                    }

                    None
                }

                let Some(module_key) = find_module_key(&game_lock) else {
                    tracing::error!(
                        url = game_lock.url,
                        title = game_lock.manifest.game.title.default_translation(),
                        "failed to find game integration module in package lock"
                    );

                    dialogs::error(
                        "Failed to find game integration module in package lock",
                        format!("Attempted to find {title} game integration module, but it's missing in the package lock. Perhaps the lock file is broken")
                    );

                    return;
                };

                let game_integration = self.runtime.get_value::<LuaTable>(module_key)
                    .transpose()
                    .map(|game_integration| {
                        game_integration.and_then(|game_integration| {
                            game_integration.raw_get::<LuaTable>("value")
                        })
                    });

                let game_integration = match game_integration {
                    Some(Ok(game_integration)) => game_integration,

                    Some(Err(err)) => {
                        tracing::error!(
                            ?err,
                            url = game_lock.url,
                            title = game_lock.manifest.game.title.default_translation(),
                            "failed to read game integration from the runtime"
                        );

                        dialogs::error(
                            format!("Failed to read {title} game integration from the runtime"),
                            err.to_string()
                        );

                        return;
                    }

                    None => {
                        tracing::error!(
                            url = game_lock.url,
                            title = game_lock.manifest.game.title.default_translation(),
                            "game integration module is missing in the runtime"
                        );

                        dialogs::error(
                            "Game integration module is missing in the runtime",
                            format!("Attempted to load {title} game integration, but integration module is missing in the packages runtime")
                        );

                        return;
                    }
                };

                let game_integration = GameIntegration::from_lua(
                    self.runtime.lua().clone(),
                    &game_integration
                );

                let game_integration = match game_integration {
                    Ok(game_integration) => Arc::new(game_integration),

                    Err(err) => {
                        tracing::error!(
                            ?err,
                            url = game_lock.url,
                            title = game_lock.manifest.game.title.default_translation(),
                            "failed to build game integration"
                        );

                        dialogs::error(
                            format!("Failed to build {title} game integration"),
                            err.to_string()
                        );

                        return;
                    }
                };

                let editions = match game_integration.get_editions(&consts::CURRENT_PLATFORM) {
                    Ok(editions) => editions,

                    Err(err) => {
                        tracing::error!(
                            ?err,
                            url = game_lock.url,
                            title = game_lock.manifest.game.title.default_translation(),
                            "failed to request game integration editions"
                        );

                        dialogs::error(
                            format!("Failed to build {title} game integration"),
                            err.to_string()
                        );

                        return;
                    }
                };

                let index = self.cards_list.guard().push_back(CardsListInit {
                    image: ImagePath::LazyLoad(game_lock.manifest.game.images.poster.clone()),
                    title: title.to_string(),
                    variants: editions.as_ref()
                        .map(|variants| {
                            variants.iter()
                                .map(|variant| {
                                    match &lang {
                                        Ok(lang) => variant.title.translate(lang),
                                        Err(_)   => variant.title.default_translation()
                                    }
                                })
                                .map(String::from)
                                .collect::<Box<[String]>>()
                        })
                });

                self.games.push(GameInfo {
                    lock: game_lock,
                    integration: game_integration,
                    editions,
                    card_index: index
                });
            }

            LibraryPageInput::SelectGameWithUrl(url) => {
                for game_info in &self.games {
                    if game_info.lock.url == url {
                        sender.input(LibraryPageInput::SelectGameWithIndex {
                            game: game_info.card_index.current_index(),
                            variant: None
                        });

                        break;
                    }
                }
            }

            LibraryPageInput::SelectGameWithIndex { game, variant } => {
                let game_info = self.games.iter()
                    .find(|game_info| game_info.card_index.current_index() == game);

                if let Some(game_info) = game_info {
                    let edition = variant.and_then(|variant| {
                        game_info.editions.as_ref()
                            .and_then(|editions| editions.get(variant))
                            .map(|edition| edition.name.clone())
                    });

                    self.game_details.emit(GameLibraryDetailsInput::SetGame {
                        manifest: game_info.lock.manifest.clone(),
                        edition,
                        integration: game_info.integration.clone(),
                        index: game
                    });
                }
            }

            LibraryPageInput::CollapseGamesExceptIndex(index) => {
                self.cards_list.broadcast(CardsListInput::HideVariantsExcept(index));
            }

            LibraryPageInput::UpdateSelectedGameInfo => {
                self.game_details.emit(GameLibraryDetailsInput::UpdateGameInfo);
            }

            // LibraryPageInput::SpawnLuauEngine { generation, validator, local_validator } => {
            //     self.games.clear();
            //     self.cards_list.guard().clear();

            //     let download_manager = self.download_manager.sender().to_owned();

            //     // TODO: we don't do this now, but in future this event could be called
            //     //       multiple times, so we would need to kill unused threads.
            //     std::thread::spawn(move || {
            //         if let Err(err) = serve_generation(sender, download_manager, generation, validator, local_validator) {
            //             tracing::error!(?err, "Failed to serve generation");
            //         }
            //     });
            // }

            // LibraryPageInput::AddGameFromGeneration { url: _, manifest, listener } => {
            //     let config = config::get();

            //     let lang = config.general.language.parse::<LanguageIdentifier>();

            //     let (send, recv) = tokio::sync::oneshot::channel();

            //     // TODO: better errors handling
            //     if let Err(err) = listener.send(SyncGameCommand::GetEditions { listener: send }) {
            //         tracing::error!(?err, "Failed to request game editions");

            //         return;
            //     }

            //     // TODO: build Arc-s here
            //     let editions = match recv.await {
            //         Ok(Ok(editions)) => editions,

            //         Ok(Err(err)) => {
            //             tracing::error!(?err, "Failed to request game editions");

            //             return;
            //         }

            //         Err(err) => {
            //             tracing::error!(?err, "Failed to request game editions");

            //             return;
            //         }
            //     };

            //     self.cards_list.guard().push_back(CardsListInit {
            //         image: ImagePath::LazyLoad(manifest.game.images.poster.clone()),

            //         title: match &lang {
            //             Ok(lang) => manifest.game.title.translate(lang).to_string(),
            //             Err(_) => manifest.game.title.default_translation().to_string()
            //         },

            //         variants: editions.as_ref().map(|editions| {
            //             editions.iter()
            //                 .map(|edition| {
            //                     match &lang {
            //                         Ok(lang) => edition.title.translate(lang).to_string(),
            //                         Err(_) => edition.title.default_translation().to_string()
            //                     }
            //                 })
            //                 .collect::<Vec<_>>()
            //         })
            //     });

            //     self.games.push(GameInfo {
            //         manifest: Arc::new(manifest),
            //         editions,
            //         listener
            //     });
            // }

            // LibraryPageInput::GameRowSelected(index) => {
            //     self.cards_list.send(index, CardsListInput::EmitClick);
            // }

            // LibraryPageInput::HideOtherGamesEditions(index) => {
            //     self.cards_list.broadcast(CardsListInput::HideVariantsExcept(index));
            // }

            // LibraryPageInput::ShowGameDetails { game, variant } => {
            //     // FIXME: don't update details page if it's already open for the given game.

            //     self.cards_list.broadcast(CardsListInput::HideVariantsExcept(game.clone()));

            //     // TODO: proper errors handling
            //     let Some(game) = self.games.get(game.current_index()) else {
            //         tracing::error!(
            //             game = game.current_index(),
            //             variant = variant.map(|variant| variant.current_index()),
            //             "Failed to read game info"
            //         );

            //         return;
            //     };

            //     let edition = match (&variant, &game.editions) {
            //         (_, None) => None,
            //         (Some(variant), Some(editions)) => editions.get(variant.current_index()),
            //         (None, Some(editions)) => editions.first()
            //     };

            //     self.game_details.emit(GameLibraryDetailsMsg::UpdateGameMetadata {
            //         manifest: game.manifest.clone(),
            //         listener: game.listener.clone(),
            //         edition: edition.cloned()
            //     });
            // }

            // LibraryPageInput::Activate => {
            //     // Update back button visibility when switching pages
            // }

            // LibraryPageInput::Call(callback) => callback(self)
        }
    }
}
