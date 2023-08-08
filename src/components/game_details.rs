use relm4::prelude::*;
use relm4::component::*;

use gtk::prelude::*;
use adw::prelude::*;

use crate::components::game_card::{
    GameCardComponent,
    GameCardComponentInput
};

use crate::windows::main::MainAppMsg;
use crate::games::GameVariant;

#[derive(Debug)]
pub struct GameDetailsComponent {
    pub game_card: AsyncController<GameCardComponent>,

    pub variant: GameVariant
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameDetailsComponentInput {
    SetVariant(GameVariant),
    EditGameCard(GameCardComponentInput)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameDetailsComponentOutput {
    
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for GameDetailsComponent {
    type Init = GameVariant;
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
                    set_label: &format!("Publisher: {}", model.variant.get_publisher())
                },

                gtk::Label {
                    set_halign: gtk::Align::Start,

                    set_margin_top: 24,

                    set_label: "Played: 4,837 hours"
                },

                gtk::Label {
                    set_halign: gtk::Align::Start,

                    set_label: "Last played: yesterday"
                },

                gtk::Box {
                    set_valign: gtk::Align::Center,

                    set_margin_top: 48,
                    set_spacing: 8,

                    gtk::Button {
                        add_css_class: "pill",
                        add_css_class: "suggested-action",

                        adw::ButtonContent {
                            set_icon_name: "media-playback-start-symbolic",
                            set_label: "Play"
                        }
                    },

                    gtk::Button {
                        add_css_class: "pill",

                        adw::ButtonContent {
                            set_icon_name: "drive-harddisk-ieee1394-symbolic",
                            set_label: "Verify"
                        }
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
                .launch(init)
                .detach(),

            variant: init
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            GameDetailsComponentInput::SetVariant(variant) => {
                self.variant = variant;

                self.game_card.emit(GameCardComponentInput::SetVariant(variant));
            }

            GameDetailsComponentInput::EditGameCard(message) => self.game_card.emit(message)
        }
    }
}
