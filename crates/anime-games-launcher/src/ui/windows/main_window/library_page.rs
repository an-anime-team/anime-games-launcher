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
use std::sync::Arc;

use relm4::prelude::*;
use adw::prelude::*;

use agl_games::api::{
    ActionsPipeline, GameComponentsGroup, GameEdition, GameIntegration,
    GameLaunchInfo, GameSettingsGroup, GameVariant
};

use crate::{consts, config, i18n};
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
struct LoadedGameInfo {
    pub package: GameLock,
    pub integration: Arc<GameIntegration>,
    pub editions: Option<Box<[GameEdition]>>,
    pub card_index: DynamicIndex
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum LibraryPageInput {
    AddGame {
        /// Unique game name (internal identifier). A game package lock filename
        /// is expected to be used, though any unique value is suitable.
        name: String,

        /// Loaded game package info.
        package: GameLock,

        /// Loaded game integration object.
        integration: Arc<GameIntegration>
    },

    SelectGame {
        /// Unique game name. A game package lock filename is expected be used.
        name: String,

        /// Optional game edition name. If possible - this edition will be
        /// selected for this game.
        edition: Option<String>
    },

    SelectGameFromList {
        /// Dynamic index of the game card on the library page.
        game_card_index: DynamicIndex,

        /// Optional dynamic index of the game edition on the library page.
        game_edition_index: Option<DynamicIndex>
    },

    SelectGameWithUrl {
        /// URL to the game integration package. This event will try to find
        /// a loaded game with the same URL and select it.
        game_package_url: String,

        /// Optional game edition name. If possible - this edition will be
        /// selected for this game.
        edition: Option<String>
    },

    CollapseGamesExceptIndex(DynamicIndex),

    UpdateSelectedGameInfo,

    ScheduleGameActionsPipeline {
        game_name: String,
        game_title: String,
        actions_pipeline: Arc<ActionsPipeline>
    },

    OpenGameComponentsWindow {
        integration: Arc<GameIntegration>,
        variant: GameVariant,
        game_name: String,
        game_title: String,
        layout: Box<[GameComponentsGroup]>
    },

    OpenGameSettingsWindow {
        integration: Arc<GameIntegration>,
        variant: GameVariant,
        layout: Box<[GameSettingsGroup]>
    },

    LaunchGame {
        game_title: String,
        game_launch_info: GameLaunchInfo
    }
}

#[derive(Debug, Clone)]
pub enum LibraryPageOutput {
    ScheduleGameActionsPipeline {
        game_name: String,
        game_title: String,
        actions_pipeline: Arc<ActionsPipeline>
    },

    OpenGameComponentsWindow {
        integration: Arc<GameIntegration>,
        variant: GameVariant,
        game_name: String,
        game_title: String,
        layout: Box<[GameComponentsGroup]>
    },

    OpenGameSettingsWindow {
        integration: Arc<GameIntegration>,
        variant: GameVariant,
        layout: Box<[GameSettingsGroup]>
    },

    LaunchGame {
        game_title: String,
        game_launch_info: GameLaunchInfo
    }
}

#[derive(Debug)]
pub struct LibraryPage {
    cards_list: AsyncFactoryVecDeque<CardsList>,
    game_details: AsyncController<GameLibraryDetails>,

    /// Table of loaded games where the key is the game name (package filename).
    games: HashMap<String, LoadedGameInfo>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for LibraryPage {
    type Init = ();
    type Input = LibraryPageInput;
    type Output = LibraryPageOutput;

    view! {
        #[root]
        gtk::Box {
            set_vexpand: true,
            set_hexpand: true,

            set_orientation: gtk::Orientation::Vertical,

            adw::StatusPage {
                set_vexpand: true,
                set_hexpand: true,

                set_icon_name: Some(consts::APP_ID),

                set_title: i18n!("no_library_games_available")
                    .unwrap_or("No games available"),

                #[watch]
                set_visible: model.cards_list.is_empty()
            },

            adw::NavigationSplitView {
                set_vexpand: true,
                set_hexpand: true,

                #[watch]
                set_visible: !model.cards_list.is_empty(),

                #[wrap(Some)]
                set_sidebar = &adw::NavigationPage {
                    set_title: i18n!("games").unwrap_or("Games"),

                    #[wrap(Some)]
                    set_child = model.cards_list.widget() {
                        add_css_class: "navigation-sidebar"
                    }
                },

                #[wrap(Some)]
                set_content = &adw::NavigationPage {
                    set_title: i18n!("details").unwrap_or("Details"),

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
                    CardsListOutput::Selected { card_index, variant_index } => {
                        LibraryPageInput::SelectGameFromList {
                            game_card_index: card_index,
                            game_edition_index: variant_index
                        }
                    }

                    CardsListOutput::HideOtherVariants(index)
                        => LibraryPageInput::CollapseGamesExceptIndex(index)
                }),

            game_details: GameLibraryDetails::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    GameLibraryDetailsOutput::ScheduleGameActionsPipeline { game_name, game_title, actions_pipeline }
                        => LibraryPageInput::ScheduleGameActionsPipeline { game_name, game_title, actions_pipeline },

                    GameLibraryDetailsOutput::OpenGameComponentsWindow { integration, variant, game_name, game_title, layout }
                        => LibraryPageInput::OpenGameComponentsWindow { integration, variant, game_name, game_title, layout },

                    GameLibraryDetailsOutput::OpenGameSettingsWindow { integration, variant, layout }
                        => LibraryPageInput::OpenGameSettingsWindow { integration, variant, layout },

                    GameLibraryDetailsOutput::LaunchGame { game_title, game_launch_info }
                        => LibraryPageInput::LaunchGame { game_title, game_launch_info }
                }),

            games: HashMap::new()
        };

        model.cards_list.widget().connect_row_selected(|_, row| {
            if let Some(row) = row {
                row.emit_activate();
            }
        });

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>
    ) {
        match msg {
            LibraryPageInput::AddGame { name, package, integration } => {
                if self.games.contains_key(&name) {
                    tracing::warn!(
                        ?name,
                        url = package.url,
                        title = package.manifest.game.title.default_translation(),
                        "attempted to load a game package for a game that is already registered"
                    );

                    return;
                }

                tracing::debug!(
                    ?name,
                    url = package.url,
                    title = package.manifest.game.title.default_translation(),
                    "loading game package"
                );

                let lang = config::get().language();

                let title = match &lang {
                    Ok(lang) => package.manifest.game.title.translate(lang),
                    Err(_) => package.manifest.game.title.default_translation()
                };

                let editions = match integration.get_editions(&consts::CURRENT_PLATFORM) {
                    Ok(Some(editions)) if editions.is_empty() => None,
                    Ok(Some(editions)) => Some(editions),
                    Ok(None) => None,

                    Err(err) => {
                        tracing::error!(
                            ?err,
                            ?name,
                            url = package.url,
                            title = package.manifest.game.title.default_translation(),
                            "failed to request game integration editions"
                        );

                        dialogs::error(
                            i18n!("failed_build_game_integration", { title => title })
                                .unwrap_or_else(|| format!("Failed to build {title} game integration")),
                            err.to_string()
                        );

                        return;
                    }
                };

                let card_index = self.cards_list.guard().push_back(CardsListInit {
                    image: ImagePath::lazy_load_card(&package.manifest.game.images.poster),
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

                self.games.insert(name, LoadedGameInfo {
                    package,
                    integration,
                    editions,
                    card_index
                });
            }

            LibraryPageInput::SelectGame { name, edition } => {
                if let Some(game_info) = self.games.get(&name) {
                    self.game_details.emit(GameLibraryDetailsInput::SetGame {
                        name,
                        manifest: game_info.package.manifest.clone(),
                        integration: game_info.integration.clone(),
                        variant: GameVariant {
                            platform: *consts::CURRENT_PLATFORM,
                            edition
                        }
                    });
                }
            }

            LibraryPageInput::SelectGameFromList {
                game_card_index,
                game_edition_index
            } => {
                let result = self.games.iter()
                    .find(|(_, game_info)| {
                        game_info.card_index.current_index()
                            == game_card_index.current_index()
                    });

                if let Some((game_name, game_info)) = result {
                    let edition = game_edition_index.and_then(|edition| {
                        game_info.editions.as_ref()
                            .and_then(|editions| editions.get(edition.current_index()))
                            .map(|edition| edition.name.clone())
                    });

                    self.game_details.emit(GameLibraryDetailsInput::SetGame {
                        name: game_name.clone(),
                        manifest: game_info.package.manifest.clone(),
                        integration: game_info.integration.clone(),
                        variant: GameVariant {
                            platform: *consts::CURRENT_PLATFORM,
                            edition
                        }
                    });
                }
            }

            LibraryPageInput::SelectGameWithUrl { game_package_url, edition } => {
                for (game_name, game_info) in &self.games {
                    if game_info.package.url == game_package_url {
                        sender.input(LibraryPageInput::SelectGame {
                            name: game_name.clone(),
                            edition
                        });

                        break;
                    }
                }
            }

            LibraryPageInput::CollapseGamesExceptIndex(index) => {
                self.cards_list.broadcast(CardsListInput::HideVariantsExcept(index));
            }

            LibraryPageInput::UpdateSelectedGameInfo => {
                self.game_details.emit(GameLibraryDetailsInput::UpdateGameInfo);
            }

            LibraryPageInput::ScheduleGameActionsPipeline {
                game_name,
                game_title,
                actions_pipeline
            } => {
                let _ = sender.output(LibraryPageOutput::ScheduleGameActionsPipeline {
                    game_name,
                    game_title,
                    actions_pipeline
                });
            }

            LibraryPageInput::OpenGameComponentsWindow {
                integration,
                variant,
                game_name,
                game_title,
                layout
            } => {
                let _ = sender.output(LibraryPageOutput::OpenGameComponentsWindow {
                    integration,
                    variant,
                    game_name,
                    game_title,
                    layout
                });
            }

            LibraryPageInput::OpenGameSettingsWindow {
                integration,
                variant,
                layout
            } => {
                let _ = sender.output(LibraryPageOutput::OpenGameSettingsWindow {
                    integration,
                    variant,
                    layout
                });
            }

            LibraryPageInput::LaunchGame { game_title, game_launch_info } => {
                let _ = sender.output(LibraryPageOutput::LaunchGame {
                    game_title,
                    game_launch_info
                });
            }
        }
    }
}
