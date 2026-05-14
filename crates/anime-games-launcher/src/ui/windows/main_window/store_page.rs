// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@vk.com>
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

use std::collections::HashMap;

use adw::prelude::*;
use relm4::prelude::*;

use agl_games::manifest::GameManifest;

use crate::{consts, config, i18n};
use crate::games::GameLock;
use crate::ui::components::lazy_picture::ImagePath;
use crate::ui::components::card::CardComponent;
use crate::ui::components::cards_grid::{CardsGrid, CardsGridOutput};
use crate::ui::components::game_store_details::{
    GameStoreDetails, GameStoreDetailsInput, GameStoreDetailsOutput
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct StoreGameInfo {
    /// Game manifest URL.
    pub manifest_url: String,

    /// Game manifest.
    pub manifest: GameManifest,

    /// Dynamic index of the game card in the UI.
    pub card_index: DynamicIndex
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorePageInput {
    AddGame {
        name: String,
        manifest_url: String,
        manifest: GameManifest
    },

    OpenGameDetails(DynamicIndex),
    CloseGameDetails,

    AddLibraryPageGame {
        name: String,
        lock: GameLock
    },

    ShowLibraryGameWithUrl(String)
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorePageOutput {
    SetShowBack(bool),
    AddLibraryPageGame {
        name: String,
        lock: GameLock
    },
    ShowLibraryGameWithUrl(String)
}

#[derive(Debug)]
pub struct StorePage {
    games_cards: AsyncFactoryVecDeque<CardsGrid>,
    game_details: AsyncController<GameStoreDetails>,

    games: HashMap<String, StoreGameInfo>,

    show_game_details: bool
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for StorePage {
    type Init = ();
    type Input = StorePageInput;
    type Output = StorePageOutput;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[transition(SlideLeftRight)]
            append = if !model.show_game_details {
                gtk::ScrolledWindow {
                    set_vexpand: true,
                    set_hexpand: true,

                    adw::Clamp {
                        set_maximum_size: 900,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,

                            set_margin_all: 16,
                            set_spacing: 16,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,

                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    add_css_class: "title-1",

                                    set_label: i18n!("games_store").unwrap_or("Store")
                                },

                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    #[watch]
                                    set_label: i18n!("loaded_games_number", { number => model.games.len() })
                                        .unwrap_or_else(|| format!("Loaded {} games", model.games.len()))
                                        .as_str()
                                }
                            },

                            adw::StatusPage {
                                set_vexpand: true,
                                set_hexpand: true,

                                set_icon_name: Some(consts::APP_ID),

                                set_title: i18n!("no_store_games_available")
                                    .unwrap_or("No games available"),

                                #[watch]
                                set_visible: model.games_cards.is_empty()
                            },

                            model.games_cards.widget() {
                                set_vexpand: true,
                                set_hexpand: true,

                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Start,

                                set_row_spacing: 16,
                                set_column_spacing: 24,

                                set_homogeneous: true,

                                set_selection_mode: gtk::SelectionMode::None
                            }
                        }
                    }
                }
            } else {
                gtk::Box {
                    model.game_details.widget(),
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
            games_cards: AsyncFactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    CardsGridOutput::Clicked(index) => StorePageInput::OpenGameDetails(index)
                }),

            game_details: GameStoreDetails::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    GameStoreDetailsOutput::AddLibraryPageGame { name, lock }
                        => StorePageInput::AddLibraryPageGame { name, lock },

                    GameStoreDetailsOutput::ShowLibraryGameWithUrl(url)
                        => StorePageInput::ShowLibraryGameWithUrl(url)

                }),

            games: HashMap::new(),

            show_game_details: false
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>
    ) {
        match msg {
            StorePageInput::AddGame { name, manifest_url, manifest } => {
                if self.games.contains_key(&name) {
                    tracing::warn!(
                        ?name,
                        ?manifest_url,
                        "attempted to add a game to the store page that is already added"
                    );

                    return;
                }

                tracing::debug!(
                    ?name,
                    ?manifest_url,
                    "adding store page game"
                );

                let config = config::get();
                let lang = config.language().ok();

                let title = match &lang {
                    Some(lang) => manifest.game.title.translate(lang),
                    None => manifest.game.title.default_translation()
                };

                let card = CardComponent::medium()
                    .with_image(ImagePath::lazy_load_card(&manifest.game.images.poster))
                    .with_title(title)
                    .with_clickable(true);

                let card_index = self.games_cards.guard()
                    .push_back(card);

                self.games.insert(name, StoreGameInfo {
                    manifest_url,
                    manifest,
                    card_index
                });
            }

            StorePageInput::OpenGameDetails(index) => {
                let result = self.games.iter()
                    .find(|(_, game_info)| {
                        game_info.card_index.current_index()
                            == index.current_index()
                    });

                let Some((game_name, game_info)) = result else {
                    tracing::warn!(
                        index = index.current_index(),
                        length = self.games.len(),
                        "trying to open details page of non-existing game"
                    );

                    return;
                };

                self.game_details.emit(GameStoreDetailsInput::SetGameInfo {
                    name: game_name.clone(),
                    manifest_url: game_info.manifest_url.clone(),
                    manifest: game_info.manifest.clone()
                });

                self.show_game_details = true;
            }

            StorePageInput::CloseGameDetails => self.show_game_details = false,

            StorePageInput::AddLibraryPageGame { name, lock } => {
                let _ = sender.output(StorePageOutput::AddLibraryPageGame {
                    name,
                    lock
                });
            }

            StorePageInput::ShowLibraryGameWithUrl(url) => {
                let _ = sender.output(StorePageOutput::ShowLibraryGameWithUrl(url));
            }
        }

        // Update back button visibility
        let _ = sender.output(StorePageOutput::SetShowBack(self.show_game_details));
    }
}
