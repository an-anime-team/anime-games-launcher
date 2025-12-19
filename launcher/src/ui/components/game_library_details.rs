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

use agl_games::manifest::GameManifest;
use agl_games::engine::{
    GameVariant,
    GameIntegration,
    ActionsPipeline,
    GameLaunchInfo,
    GameLaunchStatus,
    GameSettingsGroup
};

use crate::consts;
use crate::config;
use crate::ui::dialogs;

use super::lazy_picture::{
    LazyPictureComponent, LazyPictureComponentMsg, ImagePath
};
use super::card::{CardComponent, CardComponentInput};

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum GameLibraryDetailsInput {
    SetGame {
        manifest: GameManifest,
        edition: Option<String>,
        integration: Arc<GameIntegration>,
        index: usize
    },

    UpdateGameInfo,

    ScheduleGameActionsPipeline,
    OpenGameSettingsWindow

    // EmitLaunchGame,
    // EmitKillGame,
    // EmitInstallDiff,
    // EmitOpenSettingsWindow,

    // ScheduleRunningGameStatusCheck,

    // SendSettingsWindowMsg(GameSettingsWindowInput)
}

#[derive(Debug, Clone)]
pub enum GameLibraryDetailsOutput {
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

#[derive(Debug)]
pub struct GameLibraryDetails {
    card: AsyncController<CardComponent>,
    background: AsyncController<LazyPictureComponent>,

    game_index: usize,

    game_title: Option<String>,
    game_developer: Option<String>,
    game_publisher: Option<String>,

    game_integration: Option<Arc<GameIntegration>>,
    game_variant: Option<GameVariant>,

    game_launch_info: Option<GameLaunchInfo>,
    game_actions_pipeline: Option<Arc<ActionsPipeline>>,
    game_settings_layout: Option<Box<[GameSettingsGroup]>>,

    // is_game_scheduled: bool,
    // running_game: Option<Child>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameLibraryDetails {
    type Init = ();
    type Input = GameLibraryDetailsInput;
    type Output = GameLibraryDetailsOutput;

    view! {
        gtk::Box {
            set_vexpand: true,
            set_hexpand: true,

            set_orientation: gtk::Orientation::Vertical,

            adw::StatusPage {
                set_vexpand: true,
                set_hexpand: true,

                set_icon_name: Some(consts::APP_ID),

                set_title: "No game selected",

                #[watch]
                set_visible: model.game_title.is_none()
            },

            adw::Clamp {
                set_vexpand: true,
                set_hexpand: true,

                #[watch]
                set_visible: model.game_title.is_some(),

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    set_margin_top: 16,
                    set_spacing: 16,

                    gtk::Label {
                        set_halign: gtk::Align::Start,

                        add_css_class: "title-1",

                        #[watch]
                        set_label?: model.game_title.as_deref()
                    },

                    model.background.widget() {
                        add_css_class: "card"
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,

                        set_spacing: 12,

                        // Launch game button.
                        gtk::Button {
                            #[watch]
                            set_visible: model.game_launch_info.is_some(),

                            #[watch]
                            set_css_classes?: model.game_launch_info.as_ref()
                                .map(|info| {
                                    match info.status {
                                        GameLaunchStatus::Normal  => &["pill", "suggested-action"],
                                        GameLaunchStatus::Warning => &["pill", "warning-action"],
                                        GameLaunchStatus::Danger  => &["pill", "destructive-action"]
                                    }
                                }),

                            #[watch]
                            set_tooltip?: model.game_launch_info.as_ref()
                                .map(|info| info.hint.as_ref())
                                .and_then(|hint| {
                                    hint.as_ref()
                                        .map(|hint| {
                                            // FIXME: IO-heavy thing (there's around 6 update calls each time)
                                            let config = config::get();

                                            match config.language() {
                                                Ok(lang) => hint.translate(&lang),
                                                Err(_) => hint.default_translation()
                                            }
                                        })
                                }),

                            adw::ButtonContent {
                                set_icon_name: "media-playback-start-symbolic",

                                set_label: "Play"
                            },

                            // connect_clicked => GameLibraryDetailsInput::EmitLaunchGame
                        },

                        // // Kill game button.
                        // gtk::Button {
                        //     add_css_class: "pill",
                        //     add_css_class: "destructive-action",

                        //     #[watch]
                        //     set_visible: model.running_game.is_some(),

                        //     adw::ButtonContent {
                        //         set_icon_name: "violence-symbolic",

                        //         set_label: "Kill game"
                        //     },

                        //     connect_clicked => GameLibraryDetailsInput::EmitKillGame
                        // },

                        // Execute actions pipeline button.
                        gtk::Button {
                            #[watch]
                            set_visible: model.game_actions_pipeline.is_some(),

                            // If game can be launched AND pipeline is available
                            // then make pipeline button grey, otherwise - blue.
                            #[watch]
                            set_css_classes: if model.game_launch_info.is_some() {
                                &["pill"]
                            } else {
                                &["pill", "suggested-action"]
                            },

                            adw::ButtonContent {
                                set_icon_name: "document-save-symbolic",

                                #[watch]
                                set_label?: model.game_actions_pipeline.as_ref()
                                    .map(|pipeline| {
                                        // FIXME: IO-heavy thing (there's around 6 update calls each time)
                                        let config = config::get();

                                        match config.language() {
                                            Ok(lang) => pipeline.title().translate(&lang),
                                            Err(_) => pipeline.title().default_translation()
                                        }
                                    }),

                                #[watch]
                                set_tooltip?: model.game_actions_pipeline.as_ref()
                                    .and_then(|pipeline| pipeline.description())
                                    .map(|description| {
                                        // FIXME: IO-heavy thing (there's around 6 update calls each time)
                                        let config = config::get();

                                        match config.language() {
                                            Ok(lang) => description.translate(&lang),
                                            Err(_) => description.default_translation()
                                        }
                                    }),
                            },

                            connect_clicked => GameLibraryDetailsInput::ScheduleGameActionsPipeline
                        },

                        gtk::Button {
                            add_css_class: "pill",

                            #[watch]
                            set_visible: model.game_settings_layout.is_some(),

                            adw::ButtonContent {
                                set_icon_name: "settings-symbolic",
                                set_label: "Settings"
                            },

                            connect_clicked => GameLibraryDetailsInput::OpenGameSettingsWindow
                        }
                    }
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
            card: CardComponent::builder()
                .launch(CardComponent::medium())
                .detach(),

            background: LazyPictureComponent::builder()
                .launch(LazyPictureComponent::default())
                .detach(),

            game_index: 0,

            game_title: None,
            game_developer: None,
            game_publisher: None,

            game_integration: None,
            game_variant: None,

            game_launch_info: None,
            game_actions_pipeline: None,
            game_settings_layout: None,

            // running_game: None
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
            GameLibraryDetailsInput::SetGame {
                manifest,
                edition,
                integration,
                index
            } => {
                self.card.emit(CardComponentInput::SetImage(Some(
                    ImagePath::LazyLoad(manifest.game.images.poster.clone())
                )));

                // Little trolling. I think you can sorry me.
                let date = time::OffsetDateTime::now_utc();

                let background_image = if date.month() == time::Month::April && date.day() == 1 {
                    ImagePath::resource("images/april-fools.jpg")
                } else {
                    ImagePath::lazy_load(&manifest.game.images.background)
                };

                self.background.emit(LazyPictureComponentMsg::SetImage(Some(background_image)));

                let config = config::get();

                let lang = config.language().ok();

                let title = match &lang {
                    Some(lang) => manifest.game.title.translate(lang),
                    None => manifest.game.title.default_translation()
                };

                let developer = match &lang {
                    Some(lang) => manifest.game.developer.translate(lang),
                    None => manifest.game.developer.default_translation()
                };

                let publisher = match &lang {
                    Some(lang) => manifest.game.publisher.translate(lang),
                    None => manifest.game.publisher.default_translation()
                };

                self.game_index = index;

                self.game_title = Some(title.to_string());
                self.game_developer = Some(developer.to_string());
                self.game_publisher = Some(publisher.to_string());

                self.game_integration = Some(integration);

                self.game_variant = Some(GameVariant {
                    platform: *consts::CURRENT_PLATFORM,
                    edition
                });

                sender.input(GameLibraryDetailsInput::UpdateGameInfo);
            }

            GameLibraryDetailsInput::UpdateGameInfo => {
                if let Some(integration) = &self.game_integration
                    && let Some(variant) = &self.game_variant
                {
                    match integration.get_launch_info(variant) {
                        Ok(launch_info) => self.game_launch_info = launch_info,

                        Err(err) => {
                            self.game_launch_info = None;

                            tracing::error!(?err, "failed to request game launch info");

                            dialogs::error("Failed to request game launch info", err.to_string());
                        }
                    }

                    match integration.get_actions_pipeline(variant) {
                        Ok(pipeline) => self.game_actions_pipeline = pipeline.map(Arc::from),

                        Err(err) => {
                            self.game_actions_pipeline = None;

                            tracing::error!(?err, "failed to request game actions pipeline");

                            dialogs::error("Failed to request game actions pipeline", err.to_string());
                        }
                    }

                    match integration.get_settings_layout(variant) {
                        Ok(layout) => self.game_settings_layout = layout,

                        Err(err) => {
                            self.game_settings_layout = None;

                            tracing::error!(?err, "failed to request game settings layout");

                            dialogs::error("Failed to request game settings layout", err.to_string());
                        }
                    }
                }
            }

            GameLibraryDetailsInput::ScheduleGameActionsPipeline => {
                if let Some(game_title) = &self.game_title
                    && let Some(actions_pipeline) = &self.game_actions_pipeline
                {
                    let _ = sender.output(GameLibraryDetailsOutput::ScheduleGameActionsPipeline {
                        game_index: self.game_index,
                        game_title: game_title.clone(),
                        actions_pipeline: actions_pipeline.clone()
                    });
                }
            }

            GameLibraryDetailsInput::OpenGameSettingsWindow => {
                if let Some(variant) = &self.game_variant
                    && let Some(integration) = &self.game_integration
                    && let Some(layout) = &self.game_settings_layout
                {
                    let _ = sender.output(GameLibraryDetailsOutput::OpenGameSettingsWindow {
                        variant: variant.clone(),
                        integration: integration.clone(),
                        layout: layout.clone()
                    });
                }
            }

            // GameLibraryDetailsInput::EmitLaunchGame => {
            //     if self.running_game.is_some() {
            //         tracing::warn!("You're not allowed to launch multiple games currently");

            //         return;
            //     }

            //     if let Some(launch_info) = self.game_info.as_ref().and_then(|info| info.launch_info.as_ref()) {
            //         let mut command = &mut Command::new(&launch_info.binary);

            //         if let Some(args) = &launch_info.args {
            //             command = command.args(args);
            //         }

            //         if let Some(env) = &launch_info.env {
            //             command = command.envs(env);
            //         }

            //         // TODO: pipe stdout/stderr to a log file.

            //         tracing::info!(?command, "Launching game");

            //         match command.spawn() {
            //             Ok(child) => {
            //                 self.running_game = Some(child);

            //                 sender.input(GameLibraryDetailsInput::ScheduleRunningGameStatusCheck);
            //             }

            //             Err(err) => tracing::error!(?err, "Failed to launch game")
            //         }
            //     }
            // }

            // GameLibraryDetailsInput::EmitKillGame => {
            //     if let Some(child) = &mut self.running_game {
            //         match child.kill() {
            //             Ok(_) => self.running_game = None,

            //             Err(err) => tracing::error!(?err, "Failed to kill the game")
            //         }
            //     }
            // }

            // GameLibraryDetailsInput::ScheduleRunningGameStatusCheck => {
            //     if let Some(child) = &mut self.running_game {
            //         match child.try_wait() {
            //             Ok(Some(_)) => self.running_game = None,

            //             Ok(None) => {
            //                 tokio::spawn(async move {
            //                     tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            //                     sender.input(GameLibraryDetailsInput::ScheduleRunningGameStatusCheck)
            //                 });
            //             }

            //             Err(err) => tracing::error!(?err, "Failed to check running game status")
            //         }
            //     }
            // }

            // GameLibraryDetailsInput::EmitInstallDiff => {
            //     if let Some(listener) = self.listener.as_ref() {
            //         let (send, recv) = tokio::sync::oneshot::channel();

            //         let variant = match self.game_info.as_ref().and_then(|info| info.edition.as_ref()) {
            //             Some(edition) => GameVariant::from_edition(&edition.name),
            //             None => GameVariant::default()
            //         };

            //         let result = listener.send(SyncGameCommand::StartDiffPipeline {
            //             variant,
            //             listener: send
            //         });

            //         if let Err(err) = result {
            //             tracing::error!(?err, "Failed to request diff pipeline execution");

            //             return;
            //         }

            //         // Await pipeline execution finish and reload the game's status.
            //         tokio::spawn(async move {
            //             let _ = recv.await;

            //             sender.input(GameLibraryDetailsInput::ReloadCurrentGameInfo);
            //         });
            //     }
            // }
        }
    }
}
