use std::sync::Arc;

use adw::prelude::*;

use relm4::prelude::*;
use relm4::factory::AsyncFactoryVecDeque;

use unic_langid::LanguageIdentifier;

use crate::prelude::*;
use crate::ui::components::*;

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
    publisher: String,

    show_requirements: bool
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameDetailsPageMsg {
    SetGameInfo(Arc<GameManifest>),

    AddGameClicked
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameDetailsPage {
    type Init = ();
    type Input = GameDetailsPageMsg;
    type Output = ();

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
                        set_margin_bottom: 16,

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

                            set_spacing: 16,

                            model.carousel.widget(),

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,

                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    add_css_class: "title-4",

                                    set_text: "About"
                                },
    
                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    #[watch]
                                    set_text: &model.description
                                }
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,

                                #[watch]
                                set_visible: model.show_requirements,

                                gtk::Label {
                                    set_align: gtk::Align::Start,

                                    add_css_class: "title-4",

                                    set_text: "System Requirements",
                                },

                                model.requirements.widget(),
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Start,
                            set_spacing: 16,

                            model.card.widget(),

                            gtk::Button {
                                set_css_classes: &["suggested-action", "pill"],

                                set_label: "Add",

                                connect_clicked => GameDetailsPageMsg::AddGameClicked
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,

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
                                }
                            },

                            gtk::ScrolledWindow {
                                set_propagate_natural_height: true,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 16,

                                    model.tags.widget() {
                                        set_selection_mode: gtk::SelectionMode::None
                                    },

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
            description: String::new(),

            show_requirements: false
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            GameDetailsPageMsg::SetGameInfo(manifest) => {
                let config = config::get();

                let lang = config.general.language.parse::<LanguageIdentifier>().ok();

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

                // Set text info.
                self.title = title.to_string();
                self.description = description.to_string();
                self.developer = developer.to_string();
                self.publisher = publisher.to_string();

                // Set images.
                self.card.emit(CardComponentInput::SetImage(Some(ImagePath::lazy_load(&manifest.game.images.poster))));
                self.carousel.emit(PictureCarouselMsg::SetImages(manifest.game.images.slides.iter().map(ImagePath::lazy_load).collect()));

                // Reset general game info.
                self.tags.guard().clear();

                self.requirements.emit(HardwareRequirementsComponentMsg::Clear);

                self.show_requirements = false;

                // Update general game info.
                if let Some(info) = &manifest.info {
                    // Set game tags.
                    if let Some(tags) = &info.tags {
                        let mut guard = self.tags.guard();

                        for tag in tags {
                            guard.push_back(tag.to_owned());
                        }
                    }

                    // Set hardware requirements.
                    if let Some(requirements) = &info.hardware_requirements {
                        self.show_requirements = true;

                        self.requirements.emit(HardwareRequirementsComponentMsg::SetRequirements(requirements.clone()));
                    }
                }
            }

            GameDetailsPageMsg::AddGameClicked => {
                todo!()
            }
        }
    }
}
