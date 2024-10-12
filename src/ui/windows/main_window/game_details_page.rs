use std::sync::Arc;

use adw::prelude::*;

use relm4::prelude::*;
use relm4::factory::AsyncFactoryVecDeque;

use unic_langid::LanguageIdentifier;

use crate::prelude::*;

use crate::ui::components::*;

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
    description: String,
    developer: String,
    publisher: String
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
        adw::ClampScrollable {
            set_maximum_size: 900,
            set_margin_all: 32,

            gtk::ScrolledWindow {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: gtk::Align::Center,

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

                            gtk::Label {
                                #[watch]
                                set_text: &model.description
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

                            gtk::Label {
                                set_align: gtk::Align::Start,

                                add_css_class: "dim-label",

                                #[watch]
                                set_text: &format!("Publisher: {}", model.publisher)
                            },

                            gtk::ScrolledWindow {
                                set_propagate_natural_height: true,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 16,

                                    model.tags.widget(),

                                    adw::PreferencesGroup {
                                        set_title: "Package",

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
    }

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            card: CardComponent::builder()
                .launch(CardComponent::large())
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

            title: String::new(),
            developer: String::new(),
            publisher: String::new(),
            description: String::new()
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

                let description = match &lang {
                    Some(lang) => manifest.game.description.translate(lang),
                    None => manifest.game.description.default_translation()
                };

                let developer = match &lang {
                    Some(lang) => manifest.game.developer.translate(lang),
                    None => manifest.game.developer.default_translation()
                };

                let publisher = match &lang {
                    Some(lang) => manifest.game.publisher.translate(lang),
                    None => manifest.game.publisher.default_translation()
                };

                self.title = title.to_string();
                self.description = description.to_string();
                self.developer = developer.to_string();
                self.publisher = publisher.to_string();

                self.card.emit(CardComponentInput::SetImage(Some(ImagePath::lazy_load(&manifest.game.images.poster))));
                self.carousel.emit(PictureCarouselMsg::SetImages(manifest.game.images.slides.iter().map(ImagePath::lazy_load).collect()));

                self.requirements.emit(HardwareRequirementsComponentMsg::Clear);

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
