use gtk::prelude::*;
use relm4::prelude::*;

use crate::ui::components::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameDetailsInit {
    pub title: String,
    pub card_image: String,
    pub background_image: String
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetailsInput {
    Update(GameDetailsInit)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetailsOutput {
    
}

#[derive(Debug)]
pub struct GameDetails {
    pub card: AsyncController<CardComponent>,

    pub background_image: String,
    pub title: String
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameDetails {
    type Init = ();
    type Input = GameDetailsInput;
    type Output = GameDetailsOutput;

    view! {
        adw::Clamp {
            set_hexpand: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Overlay {
                    #[wrap(Some)]
                    set_child = &adw::Clamp {
                        set_height_request: 340,
                        set_tightening_threshold: 600,

                        gtk::Picture {
                            set_content_fit: gtk::ContentFit::Cover,

                            add_css_class: "card",

                            #[watch]
                            set_filename: Some(&model.background_image)
                        }
                    },

                    add_overlay = &gtk::Box {
                        set_margin_top: 234,
                        set_margin_start: 16,
                        set_margin_end: 16,
                        set_spacing: 16,

                        model.card.widget(),

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::End,

                            gtk::Label {
                                set_halign: gtk::Align::Start,

                                add_css_class: "title-1",

                                #[watch]
                                set_label: &model.title
                            },

                            gtk::Label {
                                set_halign: gtk::Align::Start,

                                set_label: "Hoyoverse"
                            },

                            gtk::Box {
                                set_margin_top: 16,

                                set_hexpand: true,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    gtk::Label {
                                        set_halign: gtk::Align::Start,

                                        set_label: "Played for 317 hours"
                                    },

                                    gtk::Label {
                                        set_halign: gtk::Align::Start,

                                        set_label: "Last played yesterday"
                                    }
                                },

                                gtk::Button {
                                    set_hexpand: true,

                                    set_halign: gtk::Align::End,

                                    add_css_class: "pill",
                                    add_css_class: "suggested-action",

                                    set_label: "Play"
                                }
                            }
                        }
                    }
                },

                gtk::Box {
                    set_margin_top: 130,
                    set_margin_start: 16,
                    set_margin_end: 16,
                    set_spacing: 8,

                    gtk::DropDown {
                        set_width_request: CARD_MEDIUM_SIZE.0,

                        add_css_class: "flat",

                        set_model: Some(&gtk::StringList::new(&[
                            "Global",
                            "China"
                        ]))
                    }
                },

                gtk::Box {
                    set_margin_top: 32,
                    set_spacing: 16,

                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Button {
                        set_label: "123"
                    }
                }
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, _sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            card: CardComponent::builder()
                .launch(CardComponent::medium())
                .detach(),

            background_image: String::new(),
            title: String::new()
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            GameDetailsInput::Update(init) => {
                self.card.emit(CardComponentInput::SetImage(Some(init.card_image)));

                self.background_image = init.background_image;
                self.title = init.title;
            }
        }
    }
}
