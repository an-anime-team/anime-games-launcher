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

use super::lazy_picture::ImagePath;
use super::card::{CardComponent, CardComponentInput};
use super::picture_carousel::{PictureCarousel, PictureCarouselMsg};
use super::game_tags::GameTagFactory;
use super::maintainers_row::MaintainersRowFactory;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameStoreDetailsMsg {
    SetGameInfo {
        manifest_url: String,
        manifest: GameManifest
    },

    AddGameClicked
}

#[derive(Debug)]
pub struct GameStoreDetails {
    card: AsyncController<CardComponent>,
    carousel: AsyncController<PictureCarousel>,
    tags: AsyncFactoryVecDeque<GameTagFactory>,
    maintainers: AsyncFactoryVecDeque<MaintainersRowFactory>,

    manifest_url: String,

    title: String,
    description: String,
    developer: String,
    publisher: String
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameStoreDetails {
    type Init = ();
    type Input = GameStoreDetailsMsg;
    type Output = ();

    view! {
        #[root]
        adw::ClampScrollable {
            set_maximum_size: 900,
            set_margin_all: 32,

            gtk::ScrolledWindow {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: gtk::Align::Center,

                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_margin_bottom: 16,

                        add_css_class: "title-1",

                        #[watch]
                        set_label: &model.title
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_halign: gtk::Align::Start,

                        set_spacing: 16,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_halign: gtk::Align::Center,

                            set_spacing: 16,

                            model.carousel.widget(),

                            gtk::Box {
                                set_halign: gtk::Align::Start,
                                set_orientation: gtk::Orientation::Vertical,

                                set_spacing: 8,

                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    add_css_class: "title-4",

                                    set_text: "About"
                                },

                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    set_wrap: true,

                                    #[watch]
                                    set_text: &model.description
                                }
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Start,

                            set_spacing: 16,

                            model.card.widget(),

                            gtk::Button {
                                add_css_class: "pill",
                                add_css_class: "suggested-action",

                                adw::ButtonContent {
                                    set_icon_name: "list-add-symbolic",

                                    set_label: "Add"
                                },

                                connect_clicked => GameStoreDetailsMsg::AddGameClicked
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,

                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    add_css_class: "dim-label",

                                    #[watch]
                                    set_text: &format!("Developer: {}", model.developer)
                                },

                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    add_css_class: "dim-label",

                                    #[watch]
                                    set_text: &format!("Publisher: {}", model.publisher)
                                }
                            },

                            gtk::ScrolledWindow {
                                set_propagate_natural_height: true,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 16,

                                    model.tags.widget() {
                                        set_selection_mode: gtk::SelectionMode::None
                                    },

                                    adw::PreferencesGroup {
                                        set_title: "Package",

                                        model.maintainers.widget() {
                                            set_title: "Maintainers"
                                        }
                                    }
                                }
                            }
                        }
                    }
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
            card: CardComponent::builder()
                .launch(CardComponent::large())
                .detach(),

            carousel: PictureCarousel::builder()
                .launch(())
                .detach(),

            maintainers: AsyncFactoryVecDeque::builder()
                .launch_default()
                .detach(),

            tags: AsyncFactoryVecDeque::builder()
                .launch_default()
                .detach(),

            manifest_url: String::new(),

            title: String::new(),
            developer: String::new(),
            publisher: String::new(),
            description: String::new()
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>
    ) {
        match msg {
            GameStoreDetailsMsg::SetGameInfo { manifest_url, manifest } => {
                let config = config::get();
                let lang = config.language().ok();

                let title = match &lang {
                    Some(lang) => manifest.game.title.translate(lang),
                    None => manifest.game.title.default_translation()
                };

                let description = match &lang {
                    Some(lang) => manifest.game.description.translate(lang),
                    None => manifest.game.description.default_translation()
                };

                let developer = match &lang {
                    Some(lang) => manifest.game.developer.translate(lang),
                    None => manifest.game.developer.default_translation()
                };

                let publisher = match &lang {
                    Some(lang) => manifest.game.publisher.translate(lang),
                    None => manifest.game.publisher.default_translation()
                };

                // Set text info.
                self.manifest_url = manifest_url;

                self.title = title.to_string();
                self.description = description.to_string();
                self.developer = developer.to_string();
                self.publisher = publisher.to_string();

                // Set images.
                self.card.emit(CardComponentInput::SetImage(
                    Some(ImagePath::lazy_load(&manifest.game.images.poster))
                ));

                self.carousel.emit(PictureCarouselMsg::SetImages(
                    manifest.game.images.slides.iter()
                        .map(ImagePath::lazy_load)
                        .collect()
                ));

                // Set game tags.
                let mut guard = self.tags.guard();

                guard.clear();

                for tag in &manifest.game.tags {
                    guard.push_back(*tag);
                }

                drop(guard);
            }

            GameStoreDetailsMsg::AddGameClicked => {
                tracing::info!(url = ?self.manifest_url, "add game");
            }
        }
    }
}
