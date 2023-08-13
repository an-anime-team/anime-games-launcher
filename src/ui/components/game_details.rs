use relm4::prelude::*;
use relm4::component::*;

use gtk::prelude::*;

use crate::config;

use crate::games::RunGameExt;
use crate::games::genshin::Genshin;

use crate::ui::components::game_card::{
    GameCardComponent,
    GameCardComponentInput,
    CardVariant
};

#[derive(Debug)]
pub struct GameDetailsComponent {
    pub game_card: AsyncController<GameCardComponent>,

    pub variant: CardVariant,
    pub installed: bool
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetailsComponentInput {
    SetVariant(CardVariant),
    SetInstalled(bool),
    EditGameCard(GameCardComponentInput),

    EmitDownloadGame,

    LaunchGame
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetailsComponentOutput {
    DownloadGame {
        variant: CardVariant
    },

    HideDetails,
    ShowTasksFlap,

    ShowToast {
        title: String,
        message: Option<String>
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameDetailsComponent {
    type Init = CardVariant;
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
                    set_label: model.variant.get_title()
                },

                gtk::Label {
                    set_halign: gtk::Align::Start,

                    set_margin_top: 8,

                    #[watch]
                    set_label: &format!("Developer: {}", model.variant.get_author())
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
                            add_css_class: "pill",
                            add_css_class: "suggested-action",
    
                            #[watch]
                            set_visible: model.installed,
    
                            adw::ButtonContent {
                                set_icon_name: "media-playback-start-symbolic",
                                set_label: "Play"
                            },

                            connect_clicked => GameDetailsComponentInput::LaunchGame
                        },
    
                        gtk::Button {
                            add_css_class: "pill",
    
                            #[watch]
                            set_visible: model.installed,
    
                            adw::ButtonContent {
                                set_icon_name: "drive-harddisk-ieee1394-symbolic",
                                set_label: "Verify"
                            }
                        }
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    set_margin_top: 36,

                    #[watch]
                    set_visible: !model.installed,

                    gtk::DropDown {
                        set_model: Some(&gtk::StringList::new(&[
                            "Global",
                            "China"
                        ]))
                    },

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
                        },
                    }
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            game_card: GameCardComponent::builder()
                .launch(init.clone())
                .detach(),

            variant: init,
            installed: false
        };

        model.game_card.emit(GameCardComponentInput::SetClickable(false));
        model.game_card.emit(GameCardComponentInput::SetDisplayTitle(false));

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            GameDetailsComponentInput::SetVariant(variant) => {
                self.variant = variant.clone();

                self.game_card.emit(GameCardComponentInput::SetVariant(variant));
            }

            GameDetailsComponentInput::SetInstalled(installed) => {
                self.installed = installed;

                self.game_card.emit(GameCardComponentInput::SetInstalled(installed));
            }

            GameDetailsComponentInput::EditGameCard(message) => self.game_card.emit(message),

            GameDetailsComponentInput::EmitDownloadGame => {
                sender.output(GameDetailsComponentOutput::DownloadGame {
                    variant: self.variant.clone()
                }).unwrap();

                sender.output(GameDetailsComponentOutput::HideDetails).unwrap();
                sender.output(GameDetailsComponentOutput::ShowTasksFlap).unwrap();
            }

            GameDetailsComponentInput::LaunchGame => {
                match &self.variant {
                    CardVariant::Genshin => {
                        std::thread::spawn(move || {
                            let genshin = Genshin::from(&config::get().games.genshin.to_game());

                            if let Err(err) = genshin.run() {
                                sender.output(GameDetailsComponentOutput::ShowToast {
                                    title: String::from("Failed to launch the game"),
                                    message: Some(err.to_string())
                                }).unwrap();
                            }
                        });
                    }

                    _ => unimplemented!()
                }
            }
        }
    }
}
