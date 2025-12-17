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

use relm4::prelude::*;
use adw::prelude::*;

use agl_packages::storage::Storage;
use agl_runtime::runtime::Runtime;

use crate::config;
use crate::games::GameLock;
use crate::ui::dialogs::error;
use crate::ui::components::lazy_picture::ImagePath;
use crate::ui::components::cards_list::{CardsList, CardsListInit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryPageInput {
    AddGame(GameLock),

    // SpawnLuauEngine {
    //     generation: GenerationManifest,
    //     validator: AuthorityValidator,
    //     local_validator: LocalValidator
    // },

    // AddGameFromGeneration {
    //     url: String,
    //     manifest: GameManifest,
    //     listener: UnboundedSender<SyncGameCommand>
    // },

    // Activate,

    // GameRowSelected(usize),
    // HideOtherGamesEditions(DynamicIndex),

    // ShowGameDetails {
    //     game: DynamicIndex,
    //     variant: Option<DynamicIndex>
    // },

    // Call(Box<dyn FnOnce(&mut LibraryPage) + Send>)
}

// impl std::fmt::Debug for LibraryPageInput {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             LibraryPageInput::SpawnLuauEngine { .. } => f.debug_struct("SpawnLuauEngine")
//                 .finish(),

//             LibraryPageInput::AddGameFromGeneration { url, .. } => f.debug_struct("AddGameFromGeneration")
//                 .field("url", url)
//                 .finish(),

//             LibraryPageInput::Activate => f.write_str("Activate"),

//             LibraryPageInput::GameRowSelected(idx) => f.debug_tuple("GameRowSelected")
//                 .field(idx)
//                 .finish(),

//             LibraryPageInput::HideOtherGamesEditions(idx) => f.debug_tuple("HideOtherGamesEditions")
//                 .field(idx)
//                 .finish(),

//             LibraryPageInput::ShowGameDetails { game, variant } => f.debug_struct("ShowGameDetails")
//                 .field("game", game)
//                 .field("variant", variant)
//                 .finish(),

//             LibraryPageInput::Call(_) => f.write_str("Call(..)")
//         }
//     }
// }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LibraryPageOutput {
    // SetShowBack(bool)
}

pub struct LibraryPage {
    cards_list: AsyncFactoryVecDeque<CardsList>,
    // pub game_details: AsyncController<GameLibraryDetails>,
    // pub download_manager: AsyncController<DownloadManagerWindow>,

    // main_window: Option<adw::ApplicationWindow>,
    // toast_overlay: Option<adw::ToastOverlay>,

    storage: Storage,
    runtime: Runtime,

    games: Vec<GameLock>
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

                        // connect_row_activated[sender] => move |_, row| {
                        //     sender.input(LibraryPageInput::GameRowSelected(row.index() as usize));
                        // }
                    }
                },

                #[wrap(Some)]
                set_content = &adw::NavigationPage {
                    set_title: "Details",

                    set_hexpand: true,

                    // #[wrap(Some)]
                    // set_child = model.game_details.widget(),
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            cards_list: AsyncFactoryVecDeque::builder()
                .launch_default()
                .detach(),
                // .forward(sender.input_sender(), |msg| match msg {
                //     CardsListOutput::Selected { card: game, variant }
                //         => LibraryPageInput::ShowGameDetails { game, variant },

                //     CardsListOutput::HideOtherVariants(index)
                //         => LibraryPageInput::HideOtherGamesEditions(index)
                // }),

            // game_details: GameLibraryDetails::builder()
            //     .launch(parent.clone())
            //     .detach(),

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
        _sender: AsyncComponentSender<Self>
    ) {
        match msg {
            LibraryPageInput::AddGame(game) => {
                tracing::debug!(
                    url = game.url,
                    title = game.manifest.game.title.default_translation(),
                    "loading game package"
                );

                if let Err(err) = self.runtime.load_packages(&game.lock, &self.storage) {
                    tracing::error!(
                        ?err,
                        url = game.url,
                        title = game.manifest.game.title.default_translation(),
                        "failed to load game package"
                    );

                    error("Failed to load game package", err.to_string());

                    return;
                }

                let config = config::get();

                let title = match config.language() {
                    Ok(lang) => game.manifest.game.title.translate(&lang),
                    Err(_) => game.manifest.game.title.default_translation()
                };

                self.cards_list.guard().push_back(CardsListInit {
                    image: ImagePath::LazyLoad(game.manifest.game.images.poster.clone()),
                    title: title.to_string(),
                    variants: None
                });

                self.games.push(game);
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
