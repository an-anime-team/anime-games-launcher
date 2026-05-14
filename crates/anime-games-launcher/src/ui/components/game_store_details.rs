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
        name: String,
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
    AddLibraryPageGame {
        name: String,
        lock: GameLock
    },

    ShowLibraryGameWithUrl(String)
}

#[derive(Debug)]
pub struct GameStoreDetails {
    card: AsyncController<CardComponent>,
    carousel: AsyncController<PictureCarousel>,
    tags: AsyncFactoryVecDeque<GameTagFactory>,
    maintainers: AsyncFactoryVecDeque<MaintainersRowFactory>,

    name: String,
    manifest_url: String,

    title: String,
    description: String,
    developer: String,
    publisher: String,
    agreement: Option<String>,

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

                        set_ellipsize: gtk::pango::EllipsizeMode::End,

                        add_css_class: "title-1",

                        set_selectable: true,
                        set_focusable: false,

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
                                    set_focusable: false,

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
                                    set_ellipsize: gtk::pango::EllipsizeMode::End,

                                    add_css_class: "dim-label",

                                    set_selectable: true,
                                    set_focusable: false,

                                    #[watch]
                                    set_text: i18n!("game_developer", { name => &model.developer })
                                        .unwrap_or_else(|| format!("Developer: {}", &model.developer))
                                        .as_str()
                                },

                                gtk::Label {
                                    set_align: gtk::Align::Start,
                                    set_ellipsize: gtk::pango::EllipsizeMode::End,

                                    add_css_class: "dim-label",

                                    set_selectable: true,
                                    set_focusable: false,

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

            name: String::new(),
            manifest_url: String::new(),

            title: String::new(),
            developer: String::new(),
            publisher: String::new(),
            description: String::new(),
            agreement: None,

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
            GameStoreDetailsInput::SetGameInfo {
                name,
                manifest_url,
                manifest
            } => {
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

                let agreement = manifest.game.agreement.map(|agreement| {
                    match &lang {
                        Some(lang) => agreement.translate(lang).to_string(),
                        None => agreement.default_translation().to_string()
                    }
                });

                // Set text info.
                self.name = name;
                self.manifest_url = manifest_url;

                self.title = title.to_string();
                self.description = description.to_string();
                self.developer = developer.to_string();
                self.publisher = publisher.to_string();
                self.agreement = agreement;

                // Set images.
                self.card.emit(CardComponentInput::SetImage(
                    Some(ImagePath::lazy_load_card(&manifest.game.images.poster))
                ));

                self.carousel.emit(PictureCarouselMsg::SetImages(
                    manifest.game.images.slides.iter()
                        .map(ImagePath::lazy_load_background)
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

                if let Some(agreement) = &self.agreement {
                    let dialog = adw::AlertDialog::builder()
                        .heading(
                            i18n!("game_agreement_title")
                                .unwrap_or("Game integration agreement")
                        )
                        .width_request(600)
                        .height_request(400)
                        .build();

                    relm4::view! {
                        widget = gtk::ScrolledWindow {
                            set_hexpand: true,
                            set_vexpand: true,

                            gtk::Label {
                                set_halign: gtk::Align::Fill,
                                set_justify: gtk::Justification::Fill,

                                set_hexpand: true,

                                set_wrap: true,
                                set_selectable: true,
                                set_focusable: false,
                                set_use_markup: true,

                                set_markup: &i18n!("game_agreement_message", {
                                    game_title => &self.title,
                                    agreement => agreement
                                }).unwrap_or(agreement.clone())
                            }
                        }
                    }

                    dialog.set_extra_child(Some(&widget));

                    dialog.add_responses(&[
                        ("disagree", i18n!("game_agreement_disagree").unwrap_or("Disagree")),
                        ("agree", i18n!("game_agreement_agree").unwrap_or("Agree"))
                    ]);

                    dialog.set_response_appearance(
                        "agree",
                        adw::ResponseAppearance::Suggested
                    );

                    tracing::info!(
                        url = ?self.manifest_url,
                        ?agreement,
                        "show game integration agreement"
                    );

                    let result = if let Some(window) = relm4::main_adw_application().active_window() {
                        dialog.choose_future(Some(&window)).await
                    } else {
                        dialog.choose_future(None::<&adw::Window>).await
                    };

                    if result != "agree" {
                        tracing::info!(
                            url = ?self.manifest_url,
                            "user has disagreed with the game integration agreement"
                        );

                        return;
                    }

                    else {
                        tracing::info!(
                            url = ?self.manifest_url,
                            "user has agreed with the game integration agreement"
                        );
                    }
                }

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
                    let name = self.name.clone();
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

                                let _ = sender.output(GameStoreDetailsOutput::AddLibraryPageGame {
                                    name,
                                    lock
                                });

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
