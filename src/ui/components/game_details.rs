use relm4::prelude::*;
use gtk::prelude::*;

use crate::tr;

use crate::games::metadata::LauncherMetadata;
use crate::games::integrations::standards::game::{
    Status,
    StatusSeverity
};

use crate::ui::components::game_card::{
    CardInfo,
    CardComponent,
    CardComponentInput
};

#[derive(Debug)]
pub struct GameDetailsComponent {
    pub game_card: AsyncController<CardComponent>,

    pub info: CardInfo,
    pub metadata: LauncherMetadata,

    pub installed: bool,
    pub running: bool,
    pub status: Option<Status>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetailsComponentInput {
    SetInfo(CardInfo),
    SetMetadata(LauncherMetadata),
    SetInstalled(bool),
    SetRunning(bool),
    SetStatus(Option<Status>),

    EditCard(CardComponentInput),

    EmitDownloadGame,
    EmitVerifyGame,
    EmitLaunchGame,
    EmitKillGame,
    EmitOpenAddonsManager
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetailsComponentOutput {
    HideDetails,
    ShowTasksFlap,

    DownloadGame(CardInfo),
    VerifyGame(CardInfo),
    LaunchGame(CardInfo),
    KillGame(CardInfo),
    OpenAddonsManager(CardInfo),

    ShowToast {
        title: String,
        message: Option<String>
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameDetailsComponent {
    type Init = CardInfo;
    type Input = GameDetailsComponentInput;
    type Output = GameDetailsComponentOutput;

    view! {
        #[root]
        gtk::Box {
            set_valign: gtk::Align::Center,
            set_halign: gtk::Align::Center,

            set_vexpand: true,

            model.game_card.widget(),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_valign: gtk::Align::Center,

                set_margin_start: 64,

                gtk::Label {
                    set_halign: gtk::Align::Start,

                    add_css_class: "title-1",

                    #[watch]
                    set_label: model.info.get_title()
                },

                gtk::Label {
                    set_halign: gtk::Align::Start,

                    set_margin_top: 8,

                    #[watch]
                    set_label: &tr!("details-developer", {
                        "developer" = model.info.get_developer()
                    })
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    set_margin_top: 36,

                    #[watch]
                    set_visible: model.installed,

                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        
                        // TODO: translate "hours" / "minutes" / "seconds" / "Never"

                        #[watch]
                        set_label: &tr!("details-played", {
                            "played" = model.metadata.get_total_playtime_text()
                        })
                    },

                    gtk::Label {
                        set_halign: gtk::Align::Start,

                        // TODO: translate "Today" / "Yesterday" / "Never"

                        #[watch]
                        set_label: &tr!("details-last-played", {
                            "last-played" = model.metadata.get_last_played_text()
                        })
                    },

                    gtk::Box {
                        set_valign: gtk::Align::Center,

                        set_margin_top: 36,
                        set_spacing: 8,

                        gtk::Button {
                            adw::ButtonContent {
                                set_icon_name: "media-playback-start-symbolic",
                                set_label: &tr!("details-play")
                            },

                            #[watch]
                            set_visible: !model.running,

                            #[watch]
                            set_css_classes: match &model.status {
                                Some(Status { severity: StatusSeverity::Critical, .. }) => &["pill", "destructive-action"],
                                Some(Status { severity: StatusSeverity::Warning, .. })  => &["pill", "warning-action"],
                                Some(Status { severity: StatusSeverity::None, .. })     => &["pill", "suggested-action"],

                                None => &["pill", "suggested-action"]
                            },

                            #[watch]
                            set_tooltip: match &model.status {
                                Some(Status { reason: Some(reason), .. }) => reason,
                                _ => ""
                            },

                            #[watch]
                            set_sensitive: match &model.status {
                                Some(Status { allow_launch, .. }) => *allow_launch,
                                _ => true
                            },

                            connect_clicked => GameDetailsComponentInput::EmitLaunchGame
                        },

                        gtk::Button {
                            adw::ButtonContent {
                                set_icon_name: "violence-symbolic",
                                set_label: &tr!("details-kill")
                            },

                            #[watch]
                            set_visible: model.running,

                            add_css_class: "pill",
                            add_css_class: "destructive-action",

                            connect_clicked => GameDetailsComponentInput::EmitKillGame
                        },

                        gtk::Button {
                            adw::ButtonContent {
                                set_icon_name: "drive-harddisk-ieee1394-symbolic",
                                set_label: &tr!("details-verify")
                            },

                            add_css_class: "pill",

                            #[watch]
                            set_visible: !model.running,

                            connect_clicked => GameDetailsComponentInput::EmitVerifyGame
                        }
                    },

                    gtk::Box {
                        set_valign: gtk::Align::Center,

                        set_margin_top: 16,
                        set_spacing: 8,

                        gtk::Button {
                            adw::ButtonContent {
                                set_icon_name: "folder-download-symbolic",
                                set_label: &tr!("details-manage-addons")
                            },

                            add_css_class: "pill",

                            #[watch]
                            set_visible: !model.running,

                            connect_clicked => GameDetailsComponentInput::EmitOpenAddonsManager
                        },
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    set_margin_top: 36,

                    #[watch]
                    set_visible: !model.installed,

                    gtk::Box {
                        set_valign: gtk::Align::Center,

                        set_margin_top: 36,
                        set_spacing: 8,

                        gtk::Button {
                            add_css_class: "pill",
                            add_css_class: "suggested-action",

                            adw::ButtonContent {
                                set_icon_name: "folder-download-symbolic",
                                set_label: &tr!("details-download")
                            },

                            connect_clicked => GameDetailsComponentInput::EmitDownloadGame
                        }
                    }
                }
            }
        }
    }

    async fn init(init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            game_card: CardComponent::builder()
                .launch(init.clone())
                .detach(),

            info: init,
            metadata: LauncherMetadata::default(),

            installed: false,
            running: false,
            status: None
        };

        model.game_card.emit(CardComponentInput::SetClickable(false));
        model.game_card.emit(CardComponentInput::SetDisplayTitle(false));

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            GameDetailsComponentInput::SetInfo(info) => {
                self.info = info.clone();

                self.game_card.emit(CardComponentInput::SetInfo(info));
            }

            GameDetailsComponentInput::SetMetadata(metadata) => self.metadata = metadata,

            GameDetailsComponentInput::SetInstalled(installed) => {
                self.installed = installed;

                self.game_card.emit(CardComponentInput::SetInstalled(installed));
            }

            GameDetailsComponentInput::SetRunning(running) => self.running = running,
            GameDetailsComponentInput::SetStatus(status) => self.status = status,

            GameDetailsComponentInput::EditCard(message) => self.game_card.emit(message),

            GameDetailsComponentInput::EmitDownloadGame => {
                sender.output(GameDetailsComponentOutput::DownloadGame(self.info.clone())).unwrap();

                sender.output(GameDetailsComponentOutput::HideDetails).unwrap();
                sender.output(GameDetailsComponentOutput::ShowTasksFlap).unwrap();
            }

            GameDetailsComponentInput::EmitVerifyGame => {
                sender.output(GameDetailsComponentOutput::VerifyGame(self.info.clone())).unwrap();

                sender.output(GameDetailsComponentOutput::HideDetails).unwrap();
                sender.output(GameDetailsComponentOutput::ShowTasksFlap).unwrap();
            }

            GameDetailsComponentInput::EmitLaunchGame => {
                sender.output(GameDetailsComponentOutput::LaunchGame(self.info.clone())).unwrap();
            }

            GameDetailsComponentInput::EmitKillGame => {
                sender.output(GameDetailsComponentOutput::KillGame(self.info.clone())).unwrap();
            }

            GameDetailsComponentInput::EmitOpenAddonsManager => {
                sender.output(GameDetailsComponentOutput::OpenAddonsManager(self.info.clone())).unwrap();
            }
        }
    }
}
