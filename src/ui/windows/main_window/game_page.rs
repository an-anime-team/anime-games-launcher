use adw::prelude::*;
use gtk::prelude::*;

use relm4::{factory::AsyncFactoryVecDeque, prelude::*};
use unic_langid::LanguageIdentifier;

use crate::{
    games::{
        manifest::info::{
            game_tag::GameTag,
            hardware_requirements::{
                cpu::CpuHardwareRequirements, disk::DiskHardwareRequirements, disk_type::DiskType,
                gpu::GpuHardwareRequirements, ram::RamHardwareRequirements,
                requirements::HardwareRequirements, GameHardwareRequirements,
            },
        },
        prelude::LocalizableString,
    },
    ui::components::{
        card::*, game_tags::*, maintainers_row::MaintainersRowFactory, picture_carousel::*,
        requirements::*,
    },
};

#[derive(Debug)]
pub struct GamePageAppInit {
    pub card_image: String,
    pub carousel_images: Vec<String>,
    pub title: String,
    pub developer: String,
    pub description: String,
    pub description_split: usize,
    pub tags: Vec<GameTag>,
    pub requirements: GameHardwareRequirements,
    pub version: String,
    pub repo_name: String,
    pub maintainers: Vec<String>,
}

#[derive(Debug)]
pub struct GamePageApp {
    card: AsyncController<CardComponent>,
    carousel: AsyncController<PictureCarousel>,
    title: String,
    developer: String,
    description_short: String,
    description_long: String,
    tags: AsyncFactoryVecDeque<GameTagFactory>,
    requirements: AsyncController<RequirementsComponent>,
    version: String,
    repo_name: String,
    maintainers: AsyncFactoryVecDeque<MaintainersRowFactory>,
}

#[derive(Debug)]
pub enum GamePageAppMsg {
    Add,
    Update(GamePageAppInit),
}

#[derive(Debug)]
pub enum GamePageAppOutput {
    Hide,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GamePageApp {
    type Init = ();
    type Input = GamePageAppMsg;
    type Output = GamePageAppOutput;

    view! {
        #[root]
        gtk::ScrolledWindow {
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 16,
                set_margin_all: 16,
                set_vexpand: true,

                gtk::Label {
                    #[watch]
                    set_markup: &model.title,
                    add_css_class: "title-1",
                    set_halign: gtk::Align::Start,
                },

                set_halign: gtk::Align::Center,

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
                            set_text: "About",
                            set_align: gtk::Align::Start,
                            add_css_class: "title-4",
                        },

                        gtk::TextView {
                            #[watch]
                            set_buffer: {
                                let buffer = gtk::TextBuffer::new(None);
                                buffer.set_text(&model.description_short);
                                Some(&gtk::TextBuffer::from(buffer))
                            },
                            set_wrap_mode: gtk::WrapMode::Word,
                            set_editable: false,
                            set_can_target: false,
                            set_cursor_visible: false,
                            add_css_class: "body",
                        },

                        gtk::Expander {
                            set_label: Some("Read More"),

                            gtk::TextView {
                                #[watch]
                                set_buffer: {
                                    let buffer = gtk::TextBuffer::new(None);
                                    buffer.set_text(&model.description_long);
                                    Some(&gtk::TextBuffer::from(buffer))
                                },
                                set_wrap_mode: gtk::WrapMode::Word,
                                set_editable: false,
                                set_can_target: false,
                                set_cursor_visible: false,
                                add_css_class: "body",
                            }
                        },

                        gtk::Label {
                            set_text: "System Requirements",
                            set_align: gtk::Align::Start,
                            add_css_class: "title-4",
                        },

                        model.requirements.widget(),
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_valign: gtk::Align::Start,
                        set_spacing: 16,

                        model.card.widget(),

                        gtk::Button {
                            set_label: "Add",
                            set_css_classes: &["suggested-action", "pill"],
                            connect_clicked => GamePageAppMsg::Add,
                        },

                        gtk::Label {
                            #[watch]
                            set_text: &format!("Developer: {}", model.developer),
                            set_align: gtk::Align::Start,
                            add_css_class: "dim-label",
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
                                            #[watch]
                                            set_text: &model.repo_name,
                                            add_css_class: "dim-label",
                                        }
                                    },

                                    model.maintainers.widget() {
                                        set_title: "Maintainers",
                                    },

                                    adw::ActionRow {
                                        set_title: "Version",

                                        add_suffix = &gtk::Label {
                                            #[watch]
                                            set_text: &model.version,
                                            add_css_class: "dim-label",
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

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            card: CardComponent::builder()
                .launch(CardComponent {
                    ..Default::default()
                })
                .detach(),
            carousel: PictureCarousel::builder().launch(()).detach(),
            title: String::from("N/A"),
            developer: String::from("N/A"),
            description_short: String::from("N/A"),
            description_long: String::from("N/A"),
            tags: AsyncFactoryVecDeque::builder().launch_default().detach(),
            requirements: RequirementsComponent::builder()
                .launch((
                    GameHardwareRequirements {
                        minimal: HardwareRequirements {
                            cpu: Some(CpuHardwareRequirements {
                                model: LocalizableString::Raw(String::from("N/A")),
                                cores: None,
                                frequency: None,
                            }),
                            gpu: Some(GpuHardwareRequirements {
                                model: LocalizableString::Raw(String::from("N/A")),
                                vram: None,
                            }),
                            ram: Some(RamHardwareRequirements {
                                size: 0,
                                frequency: None,
                            }),
                            disk: Some(DiskHardwareRequirements {
                                size: 0,
                                disk_type: None,
                            }),
                        },
                        optimal: None,
                    },
                    LanguageIdentifier::default(),
                ))
                .detach(),
            repo_name: String::from("N/A"),
            maintainers: AsyncFactoryVecDeque::builder().launch_default().detach(),
            version: String::from("N/A"),
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            GamePageAppMsg::Add => {
                println!("Added Game");
            }
            GamePageAppMsg::Update(init) => {
                self.card
                    .sender()
                    .send(CardComponentInput::SetImage(Some(init.card_image)))
                    .unwrap();
                self.requirements
                    .sender()
                    .send(RequirementsComponentMsg::Update((
                        init.requirements,
                        LanguageIdentifier::default(),
                    )))
                    .unwrap();
                self.carousel
                    .sender()
                    .send(PictureCarouselMsg::Update(init.carousel_images))
                    .unwrap();

                self.title = init.title;
                self.developer = init.developer;
                self.description_short = init.description;
                self.description_long = self.description_short.split_off(init.description_split);
                self.repo_name = init.repo_name;
                self.version = init.version;

                // Clear these first
                self.tags.guard().clear();
                self.maintainers.guard().clear();

                for tag in init.tags {
                    self.tags.guard().push_back(tag);
                }

                for maintainer in init.maintainers {
                    self.maintainers.guard().push_back(maintainer);
                }
            }
        }
    }
}
