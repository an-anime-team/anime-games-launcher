use relm4::prelude::*;
use gtk::prelude::*;

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

    pub installed: bool,
    pub status: Option<Status>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetailsComponentInput {
    SetInfo(CardInfo),
    SetInstalled(bool),
    SetStatus(Option<Status>),

    EditCard(CardComponentInput),

    EmitDownloadGame,
    EmitVerifyGame,
    EmitLaunchGame,
    EmitOpenAddonsManager
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetailsComponentOutput {
    HideDetails,
    ShowTasksFlap,

    DownloadGame(CardInfo),
    VerifyGame(CardInfo),
    LaunchGame(CardInfo),
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
                    set_label: &format!("Developer: {}", model.info.get_developer())
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    set_margin_top: 36,

                    #[watch]
                    set_visible: model.installed,

                    gtk::Label {
                        set_halign: gtk::Align::Start,

                        set_label: "Played: 4,837 hours"
                    },

                    gtk::Label {
                        set_halign: gtk::Align::Start,

                        set_label: "Last played: yesterday"
                    },

                    gtk::Box {
                        set_valign: gtk::Align::Center,

                        set_margin_top: 36,
                        set_spacing: 8,

                        gtk::Button {
                            adw::ButtonContent {
                                set_icon_name: "media-playback-start-symbolic",
                                set_label: "Play"
                            },

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
                            add_css_class: "pill",

                            adw::ButtonContent {
                                set_icon_name: "drive-harddisk-ieee1394-symbolic",
                                set_label: "Verify"
                            },

                            connect_clicked => GameDetailsComponentInput::EmitVerifyGame
                        }
                    },

                    gtk::Box {
                        set_valign: gtk::Align::Center,

                        set_margin_top: 16,
                        set_spacing: 8,

                        gtk::Button {
                            add_css_class: "pill",

                            adw::ButtonContent {
                                set_icon_name: "folder-download-symbolic",
                                set_label: "Manage addons"
                            },

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
                                set_label: "Download"
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

            installed: false,
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

            GameDetailsComponentInput::SetInstalled(installed) => {
                self.installed = installed;

                self.game_card.emit(CardComponentInput::SetInstalled(installed));
            }

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

                sender.output(GameDetailsComponentOutput::HideDetails).unwrap();
            }

            GameDetailsComponentInput::EmitOpenAddonsManager => {
                sender.output(GameDetailsComponentOutput::OpenAddonsManager(self.info.clone())).unwrap();
            }
        }
    }
}
