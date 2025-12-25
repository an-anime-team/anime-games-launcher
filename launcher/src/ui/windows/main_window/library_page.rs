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

use agl_games::engine::{
    GameEdition,
    GameVariant,
    GameIntegration,
    GameLaunchInfo,
    ActionsPipeline,
    GameSettingsGroup
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
struct GameInfo {
    pub package: GameLock,
    pub integration: Arc<GameIntegration>,
    pub editions: Option<Box<[GameEdition]>>,
    pub card_index: DynamicIndex
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum LibraryPageInput {
    AddGame {
        package: GameLock,
        integration: Arc<GameIntegration>
    },

    SelectGameWithUrl(String),

    SelectGameWithIndex {
        game: usize,
        variant: Option<usize>
    },

    CollapseGamesExceptIndex(DynamicIndex),

    UpdateSelectedGameInfo,

    ScheduleGameActionsPipeline {
        game_index: usize,
        game_title: String,
        actions_pipeline: Arc<ActionsPipeline>
    },

    OpenGameSettingsWindow {
        variant: GameVariant,
        integration: Arc<GameIntegration>,
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
        game_index: usize,
        game_title: String,
        actions_pipeline: Arc<ActionsPipeline>
    },

    OpenGameSettingsWindow {
        variant: GameVariant,
        integration: Arc<GameIntegration>,
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

    games: Vec<GameInfo>
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
                    set_title: i18n!("games").unwrap_or("Games"),

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
                .forward(sender.input_sender(), |msg| match msg {
                    GameLibraryDetailsOutput::ScheduleGameActionsPipeline { game_index, game_title, actions_pipeline }
                        => LibraryPageInput::ScheduleGameActionsPipeline { game_index, game_title, actions_pipeline },

                    GameLibraryDetailsOutput::OpenGameSettingsWindow { variant, integration, layout }
                        => LibraryPageInput::OpenGameSettingsWindow { variant, integration, layout },

                    GameLibraryDetailsOutput::LaunchGame { game_title, game_launch_info }
                        => LibraryPageInput::LaunchGame { game_title, game_launch_info }
                }),

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

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>
    ) {
        match msg {
            LibraryPageInput::AddGame { package, integration } => {
                tracing::debug!(
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
                    image: ImagePath::LazyLoad(package.manifest.game.images.poster.clone()),
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
                    package,
                    integration,
                    editions,
                    card_index
                });
            }

            LibraryPageInput::SelectGameWithUrl(url) => {
                for game_info in &self.games {
                    if game_info.package.url == url {
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
                        manifest: game_info.package.manifest.clone(),
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

            LibraryPageInput::ScheduleGameActionsPipeline {
                game_index,
                game_title,
                actions_pipeline
            } => {
                let _ = sender.output(LibraryPageOutput::ScheduleGameActionsPipeline {
                    game_index,
                    game_title,
                    actions_pipeline
                });
            }

            LibraryPageInput::OpenGameSettingsWindow {
                variant,
                integration,
                layout
            } => {
                let _ = sender.output(LibraryPageOutput::OpenGameSettingsWindow {
                    variant,
                    integration,
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
