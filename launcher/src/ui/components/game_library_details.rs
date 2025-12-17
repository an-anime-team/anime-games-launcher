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

use unic_langid::LanguageIdentifier;

use agl_games::engine::{
    GameEdition,
    GameLaunchStatus,
    GameLaunchInfo,
    InstallationStatus,
    GameSettingsGroup
};

use crate::consts;

use super::lazy_picture::LazyPictureComponent;
use super::card::CardComponent;

#[derive(Debug, Clone)]
pub struct GameLibraryDetailsMetadata {
    pub title: String,
    pub developer: String,
    pub publisher: String
}

#[derive(Debug, Clone)]
pub struct GameLibraryDetailsInfo {
    pub edition: Option<GameEdition>,
    pub install_status: Option<InstallationStatus>,
    pub launch_info: Option<GameLaunchInfo>,
    pub settings_layout: Option<Vec<GameSettingsGroup>>
}

#[derive(Debug)]
pub enum GameLibraryDetailsMsg {
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

#[derive(Debug)]
pub struct GameLibraryDetails {
    card: AsyncController<CardComponent>,
    background: AsyncController<LazyPictureComponent>,
    // settings_window: AsyncController<GameSettingsWindow>,

    // listener: Option<UnboundedSender<SyncGameCommand>>,
    game_metadata: Option<GameLibraryDetailsMetadata>,
    game_info: Option<GameLibraryDetailsInfo>,

    // is_loading: bool,
    // running_game: Option<Child>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameLibraryDetails {
    type Init = adw::ApplicationWindow;
    type Input = GameLibraryDetailsMsg;
    type Output = ();

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
                set_visible: model.is_loading
            },

            adw::Clamp {
                set_vexpand: true,
                set_hexpand: true,

                #[watch]
                set_visible: !model.is_loading,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    set_margin_top: 16,
                    set_spacing: 16,

                    gtk::Label {
                        set_halign: gtk::Align::Start,

                        add_css_class: "title-1",

                        #[watch]
                        set_label?: model.game_metadata.as_ref().map(|info| info.title.as_str())
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
                            set_css_classes?: model.game_info.as_ref().and_then(|info| {
                                info.launch_info.as_ref().map(|info| {
                                    match info.status {
                                        GameLaunchStatus::Normal    => &["pill", "suggested-action"],
                                        GameLaunchStatus::Warning   => &["pill", "warning-action"],
                                        GameLaunchStatus::Dangerous => &["pill", "destructive-action"],
                                        GameLaunchStatus::Disabled  => &["pill", ""]
                                    }
                                })
                            }),

                            #[watch]
                            set_visible: {
                                let game_installed = model.game_info.as_ref()
                                    .and_then(|info| info.install_status)
                                    .map(|install_status| {
                                        [
                                            InstallationStatus::Installed,
                                            InstallationStatus::UpdateAvailable
                                        ].contains(&install_status)
                                    }).unwrap_or(false);

                                model.running_game.is_none() && game_installed
                            },

                            #[watch]
                            set_sensitive?: model.game_info.as_ref().and_then(|info| {
                                info.launch_info.as_ref().map(|info| {
                                    info.status != GameLaunchStatus::Disabled
                                })
                            }),

                            #[watch]
                            set_tooltip?: model.game_info.as_ref()
                                .and_then(|info| info.launch_info.as_ref())
                                .map(|info| info.hint.as_ref())
                                .and_then(|hint| {
                                    hint.as_ref()
                                        .map(|hint| {
                                            // FIXME: IO-heavy thing (there's around 6 update calls each time)
                                            let config = config::get();

                                            let lang = config.general.language.parse::<LanguageIdentifier>();

                                            match &lang {
                                                Ok(lang) => hint.translate(lang),
                                                Err(_) => hint.default_translation()
                                            }
                                        })
                                }),

                            adw::ButtonContent {
                                set_icon_name: "media-playback-start-symbolic",

                                set_label: "Play"
                            },

                            connect_clicked => GameLibraryDetailsMsg::EmitLaunchGame
                        },

                        // Kill game button.
                        gtk::Button {
                            add_css_class: "pill",
                            add_css_class: "destructive-action",

                            #[watch]
                            set_visible: model.running_game.is_some(),

                            adw::ButtonContent {
                                set_icon_name: "violence-symbolic",

                                set_label: "Kill game"
                            },

                            connect_clicked => GameLibraryDetailsMsg::EmitKillGame
                        },

                        // Update / Install (execute diff) button.
                        gtk::Button {
                            #[watch]
                            set_css_classes?: model.game_info.as_ref()
                                .map(|info| {
                                    if info.install_status == Some(InstallationStatus::UpdateAvailable) {
                                        &["pill", ""]
                                    } else {
                                        &["pill", "suggested-action"]
                                    }
                                }),

                            #[watch]
                            set_visible: model.game_info.as_ref()
                                .map(|info| info.install_status != Some(InstallationStatus::Installed))
                                .unwrap_or(false), // false because install_status can be None so we're not ready yet

                            adw::ButtonContent {
                                set_icon_name: "document-save-symbolic",

                                #[watch]
                                set_label: {
                                    let not_installed = model.game_info.as_ref()
                                        .map(|info| info.install_status == Some(InstallationStatus::NotInstalled))
                                        .unwrap_or(true);

                                    if not_installed {
                                        "Install"
                                    } else {
                                        "Update"
                                    }
                                }
                            },

                            connect_clicked => GameLibraryDetailsMsg::EmitInstallDiff
                        },

                        gtk::Button {
                            add_css_class: "pill",

                            #[watch]
                            set_visible: model.game_info.as_ref()
                                .map(|info| info.settings_layout.is_some())
                                .unwrap_or(false),

                            adw::ButtonContent {
                                set_icon_name: "settings-symbolic",
                                set_label: "Settings"
                            },

                            connect_clicked => GameLibraryDetailsMsg::EmitOpenSettingsWindow
                        }
                    }
                }
            }
        }
    }

    async fn init(
        parent: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>
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
            //             GameSettingsWindowOutput::ReloadSettingsWindow => GameLibraryDetailsMsg::EmitOpenSettingsWindow,
            //             GameSettingsWindowOutput::ReloadGameStatus => GameLibraryDetailsMsg::ReloadCurrentGameInfo
            //         }
            //     }),

            // listener: None,
            game_metadata: None,
            game_info: None,

            // is_loading: true,
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
            // GameLibraryDetailsMsg::UpdateGameMetadata { manifest, listener, edition } => {
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

            //     sender.input(GameLibraryDetailsMsg::UpdateCurrentGameInfo(edition));
            // }

            // GameLibraryDetailsMsg::UpdateCurrentGameInfo(edition) => {
            //     if let Some(listener) = self.listener.as_ref() {
            //         sender.input(GameLibraryDetailsMsg::SetIsLoading(true));

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

            //             sender.input(GameLibraryDetailsMsg::SetGameInfo(GameLibraryDetailsInfo {
            //                 edition,
            //                 install_status: install_status.ok().flatten(),
            //                 launch_info: launch_info.ok().flatten(),
            //                 settings_layout: settings_layout.ok().flatten()
            //             }));

            //             sender.input(GameLibraryDetailsMsg::SetIsLoading(false));
            //         });
            //     }
            // }

            // GameLibraryDetailsMsg::ReloadCurrentGameInfo => {
            //     let edition = self.game_info.as_ref().and_then(|info| info.edition.clone());

            //     sender.input(GameLibraryDetailsMsg::UpdateCurrentGameInfo(edition));
            // }

            // GameLibraryDetailsMsg::SetIsLoading(is_loading) => self.is_loading = is_loading,
            // GameLibraryDetailsMsg::SetGameInfo(info) => self.game_info = Some(info),

            // GameLibraryDetailsMsg::EmitLaunchGame => {
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

            //                 sender.input(GameLibraryDetailsMsg::ScheduleRunningGameStatusCheck);
            //             }

            //             Err(err) => tracing::error!(?err, "Failed to launch game")
            //         }
            //     }
            // }

            // GameLibraryDetailsMsg::EmitKillGame => {
            //     if let Some(child) = &mut self.running_game {
            //         match child.kill() {
            //             Ok(_) => self.running_game = None,

            //             Err(err) => tracing::error!(?err, "Failed to kill the game")
            //         }
            //     }
            // }

            // GameLibraryDetailsMsg::ScheduleRunningGameStatusCheck => {
            //     if let Some(child) = &mut self.running_game {
            //         match child.try_wait() {
            //             Ok(Some(_)) => self.running_game = None,

            //             Ok(None) => {
            //                 tokio::spawn(async move {
            //                     tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            //                     sender.input(GameLibraryDetailsMsg::ScheduleRunningGameStatusCheck)
            //                 });
            //             }

            //             Err(err) => tracing::error!(?err, "Failed to check running game status")
            //         }
            //     }
            // }

            // GameLibraryDetailsMsg::EmitInstallDiff => {
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

            //             sender.input(GameLibraryDetailsMsg::ReloadCurrentGameInfo);
            //         });
            //     }
            // }

            // GameLibraryDetailsMsg::EmitOpenSettingsWindow => {
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
            //             sender.input(GameLibraryDetailsMsg::SendSettingsWindowMsg(GameSettingsWindowInput::RenderLayout {
            //                 layout,
            //                 language,
            //                 sender: listener
            //             }));

            //             sender.input(GameLibraryDetailsMsg::SendSettingsWindowMsg(GameSettingsWindowInput::EmitPresent));
            //         });
            //     }
            // }

            // GameLibraryDetailsMsg::SendSettingsWindowMsg(msg) => {
            //     self.settings_window.emit(msg);
            // }
        }
    }
}
