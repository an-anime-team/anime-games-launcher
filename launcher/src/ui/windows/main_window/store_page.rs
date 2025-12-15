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

use adw::prelude::*;
use relm4::prelude::*;

use agl_games::manifest::GameManifest;

use crate::config;
use crate::ui::components::lazy_picture::ImagePath;
use crate::ui::components::card::CardComponent;
use crate::ui::components::cards_grid::{CardsGrid, CardsGridOutput};
use crate::ui::components::game_store_details::{
    GameStoreDetails, GameStoreDetailsMsg
};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorePageInput {
    AddGame {
        manifest_url: String,
        manifest: GameManifest
    },

    OpenGameDetails(DynamicIndex),
    CloseGameDetails
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorePageOutput {
    SetShowBack(bool)
}

#[derive(Debug)]
pub struct StorePage {
    games_cards: AsyncFactoryVecDeque<CardsGrid>,
    game_details: AsyncController<GameStoreDetails>,

    games: Vec<(String, GameManifest)>,

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
                adw::ClampScrollable {
                    set_maximum_size: 900,

                    gtk::ScrolledWindow {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 16,
                            set_spacing: 16,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,

                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    add_css_class: "title-1",

                                    set_label: "Games store"
                                },

                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    #[watch]
                                    set_label: &format!("Loaded {} games", model.games.len())
                                }
                            },

                            model.games_cards.widget() {
                                set_row_spacing: 8,
                                set_column_spacing: 8,

                                set_vexpand: true,

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
                .detach(),

            games: Vec::new(),

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
            StorePageInput::AddGame { manifest_url, manifest } => {
                let config = config::get();
                let lang = config.language().ok();

                let title = match &lang {
                    Some(lang) => manifest.game.title.translate(lang),
                    None => manifest.game.title.default_translation()
                };

                let card = CardComponent::medium()
                    .with_image(ImagePath::lazy_load(&manifest.game.images.poster))
                    .with_title(title)
                    .with_clickable(true);

                self.games_cards.guard().push_back(card);

                self.games.push((manifest_url, manifest));
            }

            StorePageInput::OpenGameDetails(index) => {
                let Some((manifest_url, manifest)) = self.games.get(index.current_index()) else {
                    tracing::warn!(
                        index = index.current_index(),
                        length = self.games.len(),
                        "trying to open details page of non-existing game"
                    );

                    return;
                };

                self.game_details.emit(GameStoreDetailsMsg::SetGameInfo {
                    manifest_url: manifest_url.clone(),
                    manifest: manifest.clone()
                });

                self.show_game_details = true;
            }

            StorePageInput::CloseGameDetails => self.show_game_details = false
        }

        // Update back button visibility
        let _ = sender.output(StorePageOutput::SetShowBack(self.show_game_details));
    }
}
