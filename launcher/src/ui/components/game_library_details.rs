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
    GameIntegration,
    GameVariant,
    InstallationStatus,
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
        integration: Arc<GameIntegration>
    },

    UpdateGameInfo,

    OpenGameSettingsWindow

    // /// Set metadata for the games library details page.
    // /// This is used to render game title, pictures and other information.
    // UpdateGameMetadata {
    //     manifest: Arc<GameManifest>,
    //     listener: UnboundedSender<SyncGameCommand>,
    //     edition: Option<GameEdition>
    // },

    // /// Request game status from the integration module for the given edition.
    // UpdateCurrentGameInfo(Option<GameEdition>),

    // /// Use already fetched game status to update it once again.
    // ///
    // /// This method used the same game edition which was used the last time.
    // ReloadCurrentGameInfo,

    // SetGameInfo(GameLibraryDetailsInfo),
    // SetIsLoading(bool),

    // EmitLaunchGame,
    // EmitKillGame,
    // EmitInstallDiff,
    // EmitOpenSettingsWindow,

    // ScheduleRunningGameStatusCheck,

    // SendSettingsWindowMsg(GameSettingsWindowInput)
}

#[derive(Debug, Clone)]
pub enum GameLibraryDetailsOutput {
    OpenGameSettingsWindow {
        layout: Box<[GameSettingsGroup]>,
        integration: Arc<GameIntegration>
    }
}

#[derive(Debug)]
pub struct GameLibraryDetails {
    card: AsyncController<CardComponent>,
    background: AsyncController<LazyPictureComponent>,
    // settings_window: AsyncController<GameSettingsWindow>,

    game_title: Option<String>,
    game_developer: Option<String>,
    game_publisher: Option<String>,

    game_integration: Option<Arc<GameIntegration>>,
    game_variant: Option<GameVariant>,

    game_installation_status: Option<InstallationStatus>,
    game_launch_info: Option<GameLaunchInfo>,
    game_settings_layout: Option<Box<[GameSettingsGroup]>>,

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

                        // Play button.
                        gtk::Button {
                            #[watch]
                            set_css_classes?: model.game_launch_info.as_ref()
                                .map(|info| {
                                    match info.status {
                                        GameLaunchStatus::Normal   => &["pill", "suggested-action"],
                                        GameLaunchStatus::Warning  => &["pill", "warning-action"],
                                        GameLaunchStatus::Danger   => &["pill", "destructive-action"],
                                        GameLaunchStatus::Disabled => &["pill", ""]
                                    }
                                }),

                            #[watch]
                            set_visible: {
                                #[allow(clippy::let_and_return)]
                                let game_installed = model.game_installation_status.as_ref()
                                    .map(|install_status| {
                                        [
                                            InstallationStatus::Installed,
                                            InstallationStatus::UpdateAvailable
                                        ].contains(install_status)
                                    }).unwrap_or(false);

                                // model.running_game.is_none() &&
                                game_installed
                            },

                            #[watch]
                            set_sensitive?: model.game_launch_info.as_ref()
                                .map(|info| info.status != GameLaunchStatus::Disabled),

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

                        // Update / Install (execute diff) button.
                        gtk::Button {
                            #[watch]
                            set_css_classes?: model.game_installation_status.as_ref()
                                .map(|status| {
                                    if status == &InstallationStatus::UpdateAvailable {
                                        &["pill", ""]
                                    } else {
                                        &["pill", "suggested-action"]
                                    }
                                }),

                            #[watch]
                            set_visible: model.game_installation_status.as_ref()
                                .map(|status| status != &InstallationStatus::Installed)
                                .unwrap_or(false), // false because install_status can be None so we're not ready yet

                            adw::ButtonContent {
                                set_icon_name: "document-save-symbolic",

                                #[watch]
                                set_label: {
                                    let not_installed = model.game_installation_status.as_ref()
                                        .map(|status| status == &InstallationStatus::NotInstalled)
                                        .unwrap_or(true);

                                    if not_installed {
                                        "Install"
                                    } else {
                                        "Update"
                                    }
                                }
                            },

                            // connect_clicked => GameLibraryDetailsInput::EmitInstallDiff
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

            // settings_window: GameSettingsWindow::builder()
            //     .launch(parent)
            //     .forward(sender.input_sender(), |msg| {
            //         match msg {
            //             GameSettingsWindowOutput::ReloadSettingsWindow => GameLibraryDetailsInput::EmitOpenSettingsWindow,
            //             GameSettingsWindowOutput::ReloadGameStatus => GameLibraryDetailsInput::ReloadCurrentGameInfo
            //         }
            //     }),

            game_title: None,
            game_developer: None,
            game_publisher: None,

            game_integration: None,
            game_variant: None,

            game_installation_status: None,
            game_launch_info: None,
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
                integration
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
                    match integration.game_status(variant) {
                        Ok(status) => self.game_installation_status = Some(status),

                        Err(err) => {
                            self.game_installation_status = None;

                            tracing::error!(?err, "failed to request game installation status");

                            dialogs::error("Failed to request game installation status", err.to_string());
                        }
                    }

                    match integration.game_launch_info(variant) {
                        Ok(info) => self.game_launch_info = Some(info),

                        Err(err) => {
                            self.game_launch_info = None;

                            tracing::error!(?err, "failed to request game launch info");

                            dialogs::error("Failed to request game launch info", err.to_string());
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

            GameLibraryDetailsInput::OpenGameSettingsWindow => {
                if let Some(layout) = &self.game_settings_layout
                    && let Some(integration) = &self.game_integration
                {
                    let _ = sender.output(GameLibraryDetailsOutput::OpenGameSettingsWindow {
                        layout: layout.clone(),
                        integration: integration.clone()
                    });
                }
            }

            // GameLibraryDetailsInput::UpdateGameMetadata { manifest, listener, edition } => {
            //     let config = config::get();

            //     let lang = config.general.language.parse::<LanguageIdentifier>();

            //     let title = match &lang {
            //         Ok(lang) => manifest.game.title.translate(lang),
            //         Err(_) => manifest.game.title.default_translation()
            //     };

            //     let developer = match &lang {
            //         Ok(lang) => manifest.game.developer.translate(lang),
            //         Err(_) => manifest.game.developer.default_translation()
            //     };

            //     let publisher = match &lang {
            //         Ok(lang) => manifest.game.publisher.translate(lang),
            //         Err(_) => manifest.game.publisher.default_translation()
            //     };

            //     self.listener = Some(listener.clone());

            //     self.game_metadata = Some(GameLibraryDetailsMetadata {
            //         title: title.to_string(),
            //         developer: developer.to_string(),
            //         publisher: publisher.to_string()
            //     });

            //     self.card.emit(CardComponentInput::SetImage(Some(ImagePath::lazy_load(&manifest.game.images.poster))));

            //     // Little trolling. I think you can sorry me.
            //     let date = time::OffsetDateTime::now_utc();

            //     let image = if date.month() == time::Month::April && date.day() == 1 {
            //         tracing::info!("");
            //         tracing::info!("");
            //         tracing::info!("Happy April Fools!");
            //         tracing::info!("");
            //         tracing::info!("I hope you have a great day today ＜( ￣︿￣)");
            //         tracing::info!("");
            //         tracing::info!("");

            //         ImagePath::resource("images/april-fools.jpg")
            //     } else {
            //         ImagePath::lazy_load(&manifest.game.images.background)
            //     };

            //     self.background.emit(LazyPictureComponentMsg::SetImage(Some(image)));

            //     sender.input(GameLibraryDetailsInput::UpdateCurrentGameInfo(edition));
            // }

            // GameLibraryDetailsInput::UpdateCurrentGameInfo(edition) => {
            //     if let Some(listener) = self.listener.as_ref() {
            //         sender.input(GameLibraryDetailsInput::SetIsLoading(true));

            //         let variant = match &edition {
            //             Some(edition) => GameVariant::from_edition(&edition.name),
            //             None => GameVariant::default()
            //         };

            //         // Request game installation status update.
            //         let game_installation_status = {
            //             let listener = listener.clone();
            //             let variant = variant.clone();

            //             tokio::spawn(async move {
            //                 let (send, recv) = tokio::sync::oneshot::channel();

            //                 if let Err(err) = listener.send(SyncGameCommand::GetStatus { variant, listener: send }) {
            //                     tracing::error!(?err, "Failed to request game installation status");

            //                     return None;
            //                 }

            //                 match recv.await {
            //                     Ok(Ok(status)) => return Some(status),

            //                     Ok(Err(err)) => tracing::error!(?err, "Failed to request game installation status"),
            //                     Err(err) => tracing::error!(?err, "Failed to request game installation status")
            //                 }

            //                 None
            //             })
            //         };

            //         // Request game launching info update.
            //         let game_launch_info = {
            //             let listener = listener.clone();
            //             let variant = variant.clone();

            //             tokio::spawn(async move {
            //                 let (send, recv) = tokio::sync::oneshot::channel();

            //                 if let Err(err) = listener.send(SyncGameCommand::GetLaunchInfo { variant, listener: send }) {
            //                     tracing::error!(?err, "Failed to request game launch info");

            //                     return None;
            //                 }

            //                 match recv.await {
            //                     Ok(Ok(info)) => return Some(info),

            //                     Ok(Err(err)) => tracing::error!(?err, "Failed to request game launch info"),
            //                     Err(err) => tracing::error!(?err, "Failed to request game launch info")
            //                 }

            //                 None
            //             })
            //         };

            //         // Request game settings layout info update.
            //         let game_settings_layout = {
            //             let listener = listener.clone();
            //             let variant = variant.clone();

            //             tokio::spawn(async move {
            //                 let (send, recv) = tokio::sync::oneshot::channel();

            //                 if let Err(err) = listener.send(SyncGameCommand::GetSettingsLayout { variant, listener: send }) {
            //                     tracing::error!(?err, "Failed to request game settings layout");

            //                     return None;
            //                 }

            //                 match recv.await {
            //                     Ok(Ok(layout)) => return layout,

            //                     Ok(Err(err)) => tracing::error!(?err, "Failed to request game settings layout"),
            //                     Err(err) => tracing::error!(?err, "Failed to request game settings layout")
            //                 }

            //                 None
            //             })
            //         };

            //         tokio::spawn(async move {
            //             let (install_status, launch_info, settings_layout) = tokio::join!(
            //                 game_installation_status,
            //                 game_launch_info,
            //                 game_settings_layout
            //             );

            //             sender.input(GameLibraryDetailsInput::SetGameInfo(GameLibraryDetailsInfo {
            //                 edition,
            //                 install_status: install_status.ok().flatten(),
            //                 launch_info: launch_info.ok().flatten(),
            //                 settings_layout: settings_layout.ok().flatten()
            //             }));

            //             sender.input(GameLibraryDetailsInput::SetIsLoading(false));
            //         });
            //     }
            // }

            // GameLibraryDetailsInput::ReloadCurrentGameInfo => {
            //     let edition = self.game_info.as_ref().and_then(|info| info.edition.clone());

            //     sender.input(GameLibraryDetailsInput::UpdateCurrentGameInfo(edition));
            // }

            // GameLibraryDetailsInput::SetIsLoading(is_loading) => self.is_loading = is_loading,
            // GameLibraryDetailsInput::SetGameInfo(info) => self.game_info = Some(info),

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

            // GameLibraryDetailsInput::EmitOpenSettingsWindow => {
            //     if let Some(listener) = self.listener.as_ref() {
            //         let Some(layout) = self.game_info.as_ref().and_then(|info| info.settings_layout.clone()) else {
            //             return;
            //         };

            //         let sender = sender.clone();
            //         let listener = listener.clone();

            //         let config = config::get();

            //         let language = config.general.language.parse::<LanguageIdentifier>().ok();

            //         // Don't mind it.
            //         gtk::glib::spawn_future(async move {
            //             sender.input(GameLibraryDetailsInput::SendSettingsWindowMsg(GameSettingsWindowInput::RenderLayout {
            //                 layout,
            //                 language,
            //                 sender: listener
            //             }));

            //             sender.input(GameLibraryDetailsInput::SendSettingsWindowMsg(GameSettingsWindowInput::EmitPresent));
            //         });
            //     }
            // }

            // GameLibraryDetailsInput::SendSettingsWindowMsg(msg) => {
            //     self.settings_window.emit(msg);
            // }
        }
    }
}
