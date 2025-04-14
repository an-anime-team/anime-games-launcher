use std::sync::Arc;
use std::process::{Command, Child};

use relm4::prelude::*;
use adw::prelude::*;

use tokio::sync::mpsc::UnboundedSender;

use unic_langid::LanguageIdentifier;

use crate::prelude::*;

#[derive(Debug)]
pub enum GameLibraryDetailsMsg {
    SetGameInfo {
        manifest: Arc<GameManifest>,
        edition: GameEdition,
        listener: UnboundedSender<SyncGameCommand>
    },

    ReloadGameStatus,

    SetGameInstallationStatus(InstallationStatus),
    SetGameLaunchInfo(GameLaunchInfo),

    SetIsLoading(bool),
    SetHasSettings(bool),

    EmitLaunchGame,
    EmitKillGame,
    EmitInstallDiff,
    EmitOpenSettingsWindow,

    ScheduleRunningGameStatusCheck,

    SendSettingsWindowMsg(GameSettingsWindowInput)
}

#[derive(Debug)]
pub struct GameLibraryDetails {
    card: AsyncController<CardComponent>,
    background: AsyncController<LazyPictureComponent>,
    settings_window: AsyncController<GameSettingsWindow>,

    listener: Option<UnboundedSender<SyncGameCommand>>,

    title: Option<String>,
    developer: Option<String>,
    publisher: Option<String>,

    edition: Option<GameEdition>,
    status: Option<InstallationStatus>,
    launch_info: Option<GameLaunchInfo>,

    is_loading: bool,
    has_settings: bool,
    running_game: Option<Child>
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

                set_icon_name: Some(APP_ID),

                set_title: if model.listener.is_some() {
                    "Loading"
                } else {
                    "No game selected"
                },

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
                        set_label?: model.title.as_deref()
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
                            set_css_classes?: model.launch_info.as_ref()
                                .map(|launch_info| {
                                    match launch_info.status {
                                        GameLaunchStatus::Normal    => &["pill", "suggested-action"],
                                        GameLaunchStatus::Warning   => &["pill", "warning-action"],
                                        GameLaunchStatus::Dangerous => &["pill", "destructive-action"],
                                        GameLaunchStatus::Disabled  => &["pill", ""]
                                    }
                                }),

                            #[watch]
                            set_visible: model.running_game.is_none() && model.status.as_ref()
                                .map(|status| {
                                    [InstallationStatus::Installed, InstallationStatus::UpdateAvailable].contains(status)
                                })
                                .unwrap_or_default(),

                            #[watch]
                            set_sensitive?: model.launch_info.as_ref()
                                .map(|launch_info| launch_info.status != GameLaunchStatus::Disabled),

                            #[watch]
                            set_tooltip?: model.launch_info.as_ref()
                                .map(|launch_info| launch_info.hint.as_ref())
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
                            set_css_classes?: model.status.as_ref()
                                .map(|status| {
                                    if status == &InstallationStatus::UpdateAvailable {
                                        &["pill", ""]
                                    } else {
                                        &["pill", "suggested-action"]
                                    }
                                }),

                            #[watch]
                            set_visible: model.status != Some(InstallationStatus::Installed),

                            adw::ButtonContent {
                                set_icon_name: "document-save-symbolic",

                                #[watch]
                                set_label: if model.status == Some(InstallationStatus::NotInstalled) {
                                    "Install"
                                } else {
                                    "Update"
                                }
                            },

                            connect_clicked => GameLibraryDetailsMsg::EmitInstallDiff
                        },

                        gtk::Button {
                            add_css_class: "pill",

                            #[watch]
                            set_visible: model.has_settings,

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

    async fn init(parent: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            card: CardComponent::builder()
                .launch(CardComponent::medium())
                .detach(),

            background: LazyPictureComponent::builder()
                .launch(LazyPictureComponent::default())
                .detach(),

            settings_window: GameSettingsWindow::builder()
                .launch(parent)
                .forward(sender.input_sender(), |msg| {
                    match msg {
                        GameSettingsWindowOutput::ReloadSettingsWindow => GameLibraryDetailsMsg::EmitOpenSettingsWindow,
                        GameSettingsWindowOutput::ReloadGameStatus => GameLibraryDetailsMsg::ReloadGameStatus
                    }
                }),

            listener: None,

            title: None,
            developer: None,
            publisher: None,

            edition: None,
            status: None,
            launch_info: None,

            is_loading: true,
            has_settings: false,
            running_game: None
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            GameLibraryDetailsMsg::SetGameInfo { manifest, edition, listener } => {
                let config = config::get();

                let lang = config.general.language.parse::<LanguageIdentifier>();

                let title = match &lang {
                    Ok(lang) => manifest.game.title.translate(lang),
                    Err(_) => manifest.game.title.default_translation()
                };

                let developer = match &lang {
                    Ok(lang) => manifest.game.developer.translate(lang),
                    Err(_) => manifest.game.developer.default_translation()
                };

                let publisher = match &lang {
                    Ok(lang) => manifest.game.publisher.translate(lang),
                    Err(_) => manifest.game.publisher.default_translation()
                };

                self.listener = Some(listener.clone());

                self.title = Some(title.to_string());
                self.developer = Some(developer.to_string());
                self.publisher = Some(publisher.to_string());
                self.edition = Some(edition.clone());

                self.card.emit(CardComponentInput::SetImage(Some(ImagePath::lazy_load(&manifest.game.images.poster))));

                // Little trolling. I think you can sorry me.
                let date = time::OffsetDateTime::now_utc();

                let image = if date.month() == time::Month::April && date.day() == 1 {
                    tracing::info!("");
                    tracing::info!("");
                    tracing::info!("Happy April Fools!");
                    tracing::info!("");
                    tracing::info!("I hope you have a great day today ＜( ￣︿￣)");
                    tracing::info!("");
                    tracing::info!("");

                    ImagePath::resource("images/april-fools.jpg")
                } else {
                    ImagePath::lazy_load(&manifest.game.images.background)
                };

                self.background.emit(LazyPictureComponentMsg::SetImage(Some(image)));

                sender.input(GameLibraryDetailsMsg::ReloadGameStatus);
            }

            GameLibraryDetailsMsg::ReloadGameStatus => {
                if let (Some(listener), Some(edition)) = (self.listener.as_ref(), self.edition.as_ref()) {
                    sender.input(GameLibraryDetailsMsg::SetIsLoading(true));

                    let variant = GameVariant::from_edition(&edition.name);

                    // Request game installation status update.
                    let game_installation_status = {
                        let sender = sender.clone();
                        let listener = listener.clone();
                        let variant = variant.clone();

                        tokio::spawn(async move {
                            let (send, recv) = tokio::sync::oneshot::channel();

                            if let Err(err) = listener.send(SyncGameCommand::GetStatus { variant, listener: send }) {
                                tracing::error!(?err, "Failed to request game installation status");

                                return;
                            }

                            match recv.await {
                                Ok(Ok(status)) => {
                                    sender.input(GameLibraryDetailsMsg::SetGameInstallationStatus(status));
                                }

                                Ok(Err(err)) => tracing::error!(?err, "Failed to request game installation status"),
                                Err(err) => tracing::error!(?err, "Failed to request game installation status")
                            }
                        })
                    };

                    // Request game launching info update.
                    let game_launch_info = {
                        let sender = sender.clone();
                        let listener = listener.clone();
                        let variant = variant.clone();

                        tokio::spawn(async move {
                            let (send, recv) = tokio::sync::oneshot::channel();

                            if let Err(err) = listener.send(SyncGameCommand::GetLaunchInfo { variant, listener: send }) {
                                tracing::error!(?err, "Failed to request game launch info");

                                return;
                            }

                            match recv.await {
                                Ok(Ok(info)) => {
                                    sender.input(GameLibraryDetailsMsg::SetGameLaunchInfo(info));
                                }

                                Ok(Err(err)) => tracing::error!(?err, "Failed to request game launch info"),
                                Err(err) => tracing::error!(?err, "Failed to request game launch info")
                            }
                        })
                    };

                    // Request game settings layout info update.
                    let game_settings_layout = {
                        let sender = sender.clone();
                        let listener = listener.clone();
                        let variant = variant.clone();

                        tokio::spawn(async move {
                            let (send, recv) = tokio::sync::oneshot::channel();

                            if let Err(err) = listener.send(SyncGameCommand::GetSettingsLayout { variant, listener: send }) {
                                tracing::error!(?err, "Failed to request game settings layout");

                                return;
                            }

                            match recv.await {
                                Ok(Ok(layout)) => sender.input(GameLibraryDetailsMsg::SetHasSettings(layout.is_some())),
                                Ok(Err(err)) => tracing::error!(?err, "Failed to request game settings layout"),
                                Err(err) => tracing::error!(?err, "Failed to request game settings layout")
                            }
                        })
                    };

                    tokio::spawn(async move {
                        let _ = tokio::join!(
                            game_installation_status,
                            game_launch_info,
                            game_settings_layout
                        );

                        sender.input(GameLibraryDetailsMsg::SetIsLoading(false));
                    });
                }
            }

            GameLibraryDetailsMsg::SetGameInstallationStatus(status) => self.status = Some(status),
            GameLibraryDetailsMsg::SetGameLaunchInfo(info) => self.launch_info = Some(info),

            GameLibraryDetailsMsg::SetIsLoading(is_loading) => self.is_loading = is_loading,
            GameLibraryDetailsMsg::SetHasSettings(has_settings) => self.has_settings = has_settings,

            GameLibraryDetailsMsg::EmitLaunchGame => {
                if self.running_game.is_some() {
                    tracing::warn!("You're not allowed to launch multiple games currently");

                    return;
                }

                if let Some(launch_info) = &self.launch_info {
                    let mut command = &mut Command::new(&launch_info.binary);

                    if let Some(args) = &launch_info.args {
                        command = command.args(args);
                    }

                    if let Some(env) = &launch_info.env {
                        command = command.envs(env);
                    }

                    // TODO: pipe stdout/stderr to a log file.

                    tracing::info!(?command, "Launching game");

                    match command.spawn() {
                        Ok(child) => {
                            self.running_game = Some(child);

                            sender.input(GameLibraryDetailsMsg::ScheduleRunningGameStatusCheck);
                        }

                        Err(err) => tracing::error!(?err, "Failed to launch game")
                    }
                }
            }

            GameLibraryDetailsMsg::EmitKillGame => {
                if let Some(child) = &mut self.running_game {
                    match child.kill() {
                        Ok(_) => self.running_game = None,

                        Err(err) => tracing::error!(?err, "Failed to kill the game")
                    }
                }
            }

            GameLibraryDetailsMsg::ScheduleRunningGameStatusCheck => {
                if let Some(child) = &mut self.running_game {
                    match child.try_wait() {
                        Ok(Some(_)) => self.running_game = None,

                        Ok(None) => {
                            tokio::spawn(async move {
                                tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                                sender.input(GameLibraryDetailsMsg::ScheduleRunningGameStatusCheck)
                            });
                        }

                        Err(err) => tracing::error!(?err, "Failed to check running game status")
                    }
                }
            }

            GameLibraryDetailsMsg::EmitInstallDiff => {
                if let (Some(listener), Some(edition)) = (self.listener.as_ref(), self.edition.as_ref()) {
                    let (send, recv) = tokio::sync::oneshot::channel();

                    let result = listener.send(SyncGameCommand::StartDiffPipeline {
                        variant: GameVariant::from_edition(&edition.name),
                        listener: send
                    });

                    if let Err(err) = result {
                        tracing::error!(?err, "Failed to request diff pipeline execution");

                        return;
                    }

                    // Await pipeline execution finish and reload the game's status.
                    tokio::spawn(async move {
                        let _ = recv.await;

                        sender.input(GameLibraryDetailsMsg::ReloadGameStatus);
                    });
                }
            }

            GameLibraryDetailsMsg::EmitOpenSettingsWindow => {
                if let (Some(listener), Some(edition)) = (self.listener.as_ref(), self.edition.as_ref()) {
                    let sender = sender.clone();
                    let listener = listener.clone();

                    let variant = GameVariant::from_edition(&edition.name);

                    tokio::spawn(async move {
                        let (send, recv) = tokio::sync::oneshot::channel();

                        if let Err(err) = listener.send(SyncGameCommand::GetSettingsLayout { variant, listener: send }) {
                            tracing::error!(?err, "Failed to request game settings layout");

                            return;
                        }

                        match recv.await {
                            Ok(Ok(None)) => {
                                sender.input(GameLibraryDetailsMsg::SetHasSettings(false));
                            }

                            Ok(Ok(Some(layout))) => {
                                let config = config::get();

                                let language = config.general.language.parse::<LanguageIdentifier>().ok();

                                sender.input(GameLibraryDetailsMsg::SetHasSettings(true));

                                // Don't mind it.
                                gtk::glib::spawn_future(async move {
                                    sender.input(GameLibraryDetailsMsg::SendSettingsWindowMsg(GameSettingsWindowInput::RenderLayout {
                                        layout,
                                        language,
                                        sender: listener
                                    }));

                                    sender.input(GameLibraryDetailsMsg::SendSettingsWindowMsg(GameSettingsWindowInput::EmitPresent));
                                });
                            }

                            Ok(Err(err)) => tracing::error!(?err, "Failed to request game settings layout"),
                            Err(err) => tracing::error!(?err, "Failed to request game settings layout")
                        }
                    });
                }
            }

            GameLibraryDetailsMsg::SendSettingsWindowMsg(msg) => {
                self.settings_window.emit(msg);
            }
        }
    }
}
