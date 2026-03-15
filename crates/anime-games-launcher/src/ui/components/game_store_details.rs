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

use adw::prelude::*;
use relm4::prelude::*;

use agl_core::tasks;
use agl_packages::storage::Storage;
use agl_games::manifest::GameManifest;

use crate::{config, i18n, games};
use crate::games::GameLock;
use crate::ui::dialogs;

use super::lazy_picture::ImagePath;
use super::card::{CardComponent, CardComponentInput, CardSize};
use super::picture_carousel::{PictureCarousel, PictureCarouselMsg};
use super::game_tags::GameTagFactory;
use super::maintainers_row::MaintainersRowFactory;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameStatus {
    NotAdded,
    Adding,
    Added
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameStoreDetailsInput {
    SetGameInfo {
        manifest_url: String,
        manifest: GameManifest
    },

    SetGameStatus(GameStatus),
    UpdateGameStatus,

    EmitClick
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameStoreDetailsOutput {
    AddLibraryPageGame(GameLock),
    ShowLibraryGameWithUrl(String)
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
    publisher: String,

    status: GameStatus
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameStoreDetails {
    type Init = ();
    type Input = GameStoreDetailsInput;
    type Output = GameStoreDetailsOutput;

    view! {
        #[root]
        gtk::ScrolledWindow {
            adw::Clamp {
                set_maximum_size: 900,
                set_margin_all: 32,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: gtk::Align::Center,

                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        set_margin_bottom: 16,

                        add_css_class: "title-1",

                        set_selectable: true,

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
                                    set_halign: gtk::Align::Start,

                                    add_css_class: "title-4",

                                    set_text: i18n!("about_game")
                                        .unwrap_or("About game")
                                },

                                gtk::Label {
                                    set_hexpand: true,

                                    set_wrap: true,
                                    set_selectable: true,

                                    set_halign: gtk::Align::Fill,
                                    set_justify: gtk::Justification::Fill,

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
                                #[watch]
                                set_css_classes: match model.status {
                                    GameStatus::NotAdded => &["pill", "suggested-action"],
                                    GameStatus::Adding   => &["pill"],
                                    GameStatus::Added    => &["pill"]
                                },

                                #[watch]
                                set_sensitive: model.status != GameStatus::Adding,

                                adw::ButtonContent {
                                    #[watch]
                                    set_icon_name: match model.status {
                                        GameStatus::NotAdded => "list-add-symbolic",
                                        GameStatus::Adding   => "document-save-symbolic",
                                        GameStatus::Added    => "input-gaming-symbolic"
                                    },

                                    #[watch]
                                    set_label: &{
                                        match model.status {
                                            GameStatus::NotAdded => i18n!("game_add_to_library")
                                                .unwrap_or("Add to library")
                                                .to_string(),

                                            GameStatus::Adding => i18n!("game_adding_to_library")
                                                .unwrap_or("Addding to library...")
                                                .to_string(),

                                            GameStatus::Added => i18n!("open_game_in_library")
                                                .unwrap_or("Open in library")
                                                .to_string()
                                        }
                                    }
                                },

                                connect_clicked => GameStoreDetailsInput::EmitClick
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,

                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    add_css_class: "dim-label",

                                    set_selectable: true,

                                    #[watch]
                                    set_text: i18n!("game_developer", { name => &model.developer })
                                        .unwrap_or_else(|| format!("Developer: {}", &model.developer))
                                        .as_str()
                                },

                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    add_css_class: "dim-label",

                                    set_selectable: true,

                                    #[watch]
                                    set_text: i18n!("game_publisher", { name => &model.publisher })
                                        .unwrap_or_else(|| format!("Publisher: {}", &model.publisher))
                                        .as_str()
                                }
                            },

                            adw::Clamp {
                                set_maximum_size: CardSize::Large.width(),

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    set_spacing: 16,
                                    set_vexpand: true,

                                    adw::PreferencesGroup {
                                        set_vexpand: true,

                                        #[watch]
                                        set_visible: !model.tags.is_empty(),

                                        model.tags.widget() {
                                            set_halign: gtk::Align::Start,
                                            set_selection_mode: gtk::SelectionMode::None
                                        }
                                    },

                                    adw::PreferencesGroup {
                                        set_vexpand: true,

                                        set_title: i18n!("game_package")
                                            .unwrap_or("Package"),

                                        #[watch]
                                        set_visible: !model.maintainers.is_empty(),

                                        model.maintainers.widget() {
                                            set_title: i18n!("game_package_maintainers")
                                                .unwrap_or("Maintainers")
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
            description: String::new(),

            status: GameStatus::NotAdded
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
            GameStoreDetailsInput::SetGameInfo { manifest_url, manifest } => {
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

                // Set game package maintainers.
                let mut guard = self.maintainers.guard();

                guard.clear();

                for maintainer in &manifest.maintainers {
                    let maintainer = match &lang {
                        Some(lang) => maintainer.translate(lang),
                        None => maintainer.default_translation()
                    };

                    guard.push_back(maintainer.to_string());
                }

                drop(guard);

                // Update game status.
                sender.input(GameStoreDetailsInput::UpdateGameStatus);
            }

            GameStoreDetailsInput::SetGameStatus(status) => self.status = status,

            GameStoreDetailsInput::UpdateGameStatus => {
                let config = config::get();

                let path = config.games_path.join(games::get_name(&self.manifest_url));

                if path.is_file() {
                    self.status = GameStatus::Added;
                } else {
                    self.status = GameStatus::NotAdded;
                }
            }

            GameStoreDetailsInput::EmitClick if self.status == GameStatus::NotAdded => {
                tracing::info!(url = ?self.manifest_url, "add game");

                self.status = GameStatus::Adding;

                let config = config::get();

                let storage = match Storage::open(config.packages_resources_path) {
                    Ok(storage) => storage,
                    Err(err) => {
                        tracing::error!(?err, "failed to open packages storage");

                        return;
                    }
                };

                {
                    let url = self.manifest_url.clone();

                    tasks::spawn(async move {
                        tracing::debug!(?url, "download game package");

                        match GameLock::download(&url, &storage).await {
                            Ok(lock) => {
                                sender.input(GameStoreDetailsInput::SetGameStatus(GameStatus::Added));

                                let config = config::get();

                                let path = config.games_path.join(games::get_name(&lock.url));

                                tracing::info!(?url, ?path, "game added");

                                let lock_bytes = match serde_json::to_vec_pretty(&lock.to_json()) {
                                    Ok(lock) => lock,

                                    Err(err) => {
                                        sender.input(GameStoreDetailsInput::SetGameStatus(GameStatus::NotAdded));

                                        tracing::error!(?err, "failed to serialize game package lock");

                                        dialogs::error(
                                            i18n!("failed_serialize_game_package_lock")
                                                .unwrap_or("Failed to serialize game package lock"),
                                            err
                                        );

                                        return;
                                    }
                                };

                                if let Err(err) = std::fs::write(path, lock_bytes) {
                                    sender.input(GameStoreDetailsInput::SetGameStatus(GameStatus::NotAdded));

                                    tracing::error!(?err, "failed to save game package lock");

                                    dialogs::error(
                                        i18n!("failed_save_game_package_lock")
                                            .unwrap_or("Failed to save game package lock"),
                                        err
                                    );

                                    return;
                                }

                                let _ = sender.output(GameStoreDetailsOutput::AddLibraryPageGame(lock));

                                sender.input(GameStoreDetailsInput::UpdateGameStatus);
                            }

                            Err(err) => {
                                sender.input(GameStoreDetailsInput::SetGameStatus(GameStatus::NotAdded));

                                tracing::error!(?err, "failed to download game package");

                                dialogs::error(
                                    i18n!("failed_download_game_package")
                                        .unwrap_or("Failed to download game package"),
                                    err
                                );
                            }
                        }
                    });
                }
            }

            GameStoreDetailsInput::EmitClick if self.status == GameStatus::Added => {
                tracing::info!(url = ?self.manifest_url, "open game");

                let _ = sender.output(GameStoreDetailsOutput::ShowLibraryGameWithUrl(
                    self.manifest_url.clone()
                ));
            }

            _ => ()
        }
    }
}
