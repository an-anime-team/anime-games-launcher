use std::sync::Arc;

use gtk::prelude::*;
use adw::prelude::*;

use relm4::prelude::*;
use relm4::factory::AsyncFactoryVecDeque;

use unic_langid::LanguageIdentifier;

use crate::prelude::*;

use crate::ui::components::{
    card::*, game_tags::*, maintainers_row::MaintainersRowFactory, picture_carousel::*,
    hardware_requirements::*,
};

#[derive(Debug)]
pub struct GameDetailsPage {
    card: AsyncController<CardComponent>,
    carousel: AsyncController<PictureCarousel>,
    maintainers: AsyncFactoryVecDeque<MaintainersRowFactory>,
    tags: AsyncFactoryVecDeque<GameTagFactory>,
    requirements: AsyncController<HardwareRequirementsComponent>,

    title: String,
    developer: String,
    description_short: String,
    description_long: String,
    repo_name: String
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetailsPageInput {
    SetGameInfo(Arc<GameManifest>),

    AddGameClicked
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameDetailsPageOutput {
    Hide
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameDetailsPage {
    type Init = ();
    type Input = GameDetailsPageInput;
    type Output = GameDetailsPageOutput;

    view! {
        #[root]
        gtk::ScrolledWindow {
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_halign: gtk::Align::Center,

                set_spacing: 16,
                set_margin_all: 16,

                set_vexpand: true,

                gtk::Label {
                    set_halign: gtk::Align::Start,

                    add_css_class: "title-1",

                    #[watch]
                    set_label: &model.title
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Start,

                    set_spacing: 16,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,

                        set_spacing: 8,

                        model.carousel.widget(),

                        gtk::Label {
                            set_align: gtk::Align::Start,

                            add_css_class: "title-4",

                            set_text: "About"
                        },

                        gtk::TextView {
                            add_css_class: "body",

                            set_wrap_mode: gtk::WrapMode::Word,

                            set_editable: false,
                            set_can_target: false,
                            set_cursor_visible: false,

                            #[watch]
                            set_buffer: Some(&{
                                let buffer = gtk::TextBuffer::new(None);

                                buffer.set_text(&model.description_short);

                                buffer
                            })
                        },

                        gtk::Expander {
                            set_label: Some("Read More"),

                            gtk::TextView {
                                add_css_class: "body",

                                set_wrap_mode: gtk::WrapMode::Word,

                                set_editable: false,
                                set_can_target: false,
                                set_cursor_visible: false,

                                #[watch]
                                set_buffer: Some(&{
                                    let buffer = gtk::TextBuffer::new(None);

                                    buffer.set_text(&model.description_long);

                                    buffer
                                })
                            }
                        },

                        gtk::Label {
                            set_align: gtk::Align::Start,

                            add_css_class: "title-4",

                            set_text: "System Requirements",
                        },

                        model.requirements.widget(),
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_valign: gtk::Align::Start,
                        set_spacing: 16,

                        model.card.widget(),

                        gtk::Button {
                            set_css_classes: &["suggested-action", "pill"],

                            set_label: "Add",

                            connect_clicked => GameDetailsPageInput::AddGameClicked
                        },

                        gtk::Label {
                            set_align: gtk::Align::Start,

                            add_css_class: "dim-label",

                            #[watch]
                            set_text: &format!("Developer: {}", model.developer)
                        },

                        gtk::ScrolledWindow {
                            set_propagate_natural_height: true,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 16,

                                model.tags.widget(),

                                adw::PreferencesGroup {
                                    set_title: "Package",

                                    adw::ActionRow {
                                        set_title: "Repository",

                                        add_suffix = &gtk::Label {
                                            add_css_class: "dim-label",

                                            #[watch]
                                            set_text: &model.repo_name
                                        }
                                    },

                                    model.maintainers.widget() {
                                        set_title: "Maintainers"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            card: CardComponent::builder()
                .launch(CardComponent::default())
                .detach(),

            carousel: PictureCarousel::builder()
                .launch(())
                .detach(),

            maintainers: AsyncFactoryVecDeque::builder()
                .launch_default()
                .detach(),

            tags: AsyncFactoryVecDeque::builder()
                .launch_default()
                .detach(),

            requirements: HardwareRequirementsComponent::builder()
                .launch(())
                .detach(),

            title: String::from("N/A"),
            developer: String::from("N/A"),
            description_short: String::from("N/A"),
            description_long: String::from("N/A"),
            repo_name: String::from("N/A")
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            GameDetailsPageInput::SetGameInfo(manifest) => {
                let config = config::get();

                let lang = config.general.language.parse::<LanguageIdentifier>().ok();

                // TODO: handle images

                let title = match &lang {
                    Some(lang) => manifest.game.title.translate(lang),
                    None => manifest.game.title.default_translation()
                };

                self.card.emit(CardComponentInput::SetTitle(Some(title.to_string())));

                // TODO: clear components

                if let Some(info) = &manifest.info {
                    if let Some(requirements) = &info.hardware_requirements {
                        self.requirements.emit(HardwareRequirementsComponentMsg::SetRequirements(requirements.clone()));
                    }
                }
            }

            GameDetailsPageInput::AddGameClicked => {
                todo!()
            }
        }
    }
}
