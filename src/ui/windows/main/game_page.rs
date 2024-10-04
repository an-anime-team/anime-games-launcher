use adw::prelude::*;
use gtk::prelude::*;

use relm4::{factory::AsyncFactoryVecDeque, prelude::*};

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
        card::CardComponent, game_tags::*, maintainers_row::MaintainersRowFactory,
        requirements::RequirementsComponent,
    },
};

#[derive(Debug)]
pub struct GamePageApp {
    card: AsyncController<CardComponent>,
    title: String,
    developer: String,
    description_short: String,
    description_long: String,
    tags: AsyncFactoryVecDeque<GameTagFactory>,
    requirements: AsyncController<RequirementsComponent>,
    maintainers: AsyncFactoryVecDeque<MaintainersRowFactory>,
}

#[derive(Debug)]
pub enum GamePageAppMsg {
    Add,
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

                        #[name = "carousel"]
                        adw::Carousel {
                            set_height_request: CardComponent::default().height,

                            gtk::Picture {
                                set_filename: Some(&format!("{}1.png", TEST_PATH)),
                            },

                            gtk::Picture {
                                set_filename: Some(&format!("{}2.png", TEST_PATH)),
                            },

                            gtk::Picture {
                                set_filename: Some(&format!("{}3.png", TEST_PATH)),
                            }
                        },

                        adw::CarouselIndicatorLines {
                            set_carousel: Some(&carousel),
                        },

                        gtk::Label {
                            set_text: "About",
                            set_align: gtk::Align::Start,
                            add_css_class: "title-4",
                        },

                        gtk::TextView {
                            set_buffer: Some(&short_buffer),
                            set_wrap_mode: gtk::WrapMode::Word,
                            set_editable: false,
                            set_can_target: false,
                            set_cursor_visible: false,
                            add_css_class: "body",
                        },

                        gtk::Expander {
                            set_label: Some("Read More"),

                            gtk::TextView {
                                set_buffer: Some(&long_buffer),
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
                                            set_text: "an-anime-team",
                                            add_css_class: "dim-label",
                                        }
                                    },

                                    model.maintainers.widget() {
                                        set_title: "Maintainers",
                                    },

                                    adw::ActionRow {
                                        set_title: "Version",

                                        add_suffix = &gtk::Label {
                                            set_text: "69.42.0",
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
        let TEST_PATH = "/temp/";

        let mut model = Self {
            card: CardComponent::builder()
                .launch(CardComponent {
                    image: Some(String::from("card.jpg")),
                    ..Default::default()
                })
                .detach(),
            title: String::from("Genshin Impact"),
            developer: String::from("MiHoYo"),
            description_short: String::from("Step into Teyvat, a vast world teeming with life and flowing with elemental energy.

You and your sibling arrived here from another world. Separated by an unknown god, stripped of your powers, and cast into a deep slumber, you now awake to a world very different from when you first arrived.
Thus begins your journey across Teyvat to seek answers from The Seven — the gods of each element. Along the way, prepare to explore every inch of this wondrous world, join forces with a diverse range of characters, and unravel the countless mysteries that Teyvat holds..."),
            description_long: String::from("MASSIVE OPEN WORLD
Climb any mountain, swim across any river, and glide over the world below, taking in the jaw-dropping scenery each step of the way. And if you stop to investigate a wandering Seelie or strange mechanism, who knows what you might discover?

ELEMENTAL COMBAT SYSTEM
Harness the seven elements to unleash elemental reactions. Anemo, Electro, Hydro, Pyro, Cryo, Dendro, and Geo interact in all sorts of ways, and Vision wielders have the power to turn this to their advantage.
Will you vaporize Hydro with Pyro, electro-charge it with Electro, or freeze it with Cryo? Your mastery of the elements will give you the upper hand in battle and exploration.

BEAUTIFUL VISUALS
Feast your eyes on the world around you, with a stunning art style, real-time rendering, and finely tuned character animations delivering you a truly immersive visual experience. Lighting and weather all change naturally over time, bringing every detail of this world to life.

SOOTHING SOUNDTRACK
Let the beautiful sounds of Teyvat draw you in as you explore the expansive world around you. Performed by the world's top orchestras such as London Philharmonic Orchestra and Shanghai Symphony Orchestra, the soundtrack changes seamlessly with the time and gameplay to match the mood.

BUILD YOUR DREAM TEAM
Team up with a diverse cast of characters in Teyvat, each with their own unique personalities, stories, and abilities. Discover your favorite party combinations and level up your characters to help you conquer even the most daunting of enemies and domains.

JOURNEY WITH FRIENDS
Team up with friends across various platforms to trigger more elemental action, tackle tricky boss fights, and conquer challenging domains together to reap rich rewards.
As you stand atop the peaks of Jueyun Karst and take in the rolling clouds and vast terrain stretching out before you, you may wish to stay in Teyvat a little longer... But until you are reunited with your lost sibling, how can you rest? Go forth, Traveler, and begin your adventure!

SUPPORT
If you encounter any issues during the game, you can send us feedback via the in-game Customer Service Center.
Customer Service Email: genshin_cs@hoyoverse.com
Official Site: https://genshin.hoyoverse.com/
Forums: https://www.hoyolab.com/
Facebook: https://www.facebook.com/Genshinimpact/
Instagram: https://www.instagram.com/genshinimpact/
Twitter: https://twitter.com/GenshinImpact
YouTube: http://www.youtube.com/c/GenshinImpact
Discord: https://discord.gg/genshinimpact
Reddit: https://www.reddit.com/r/Genshin_Impact/"),
            tags: AsyncFactoryVecDeque::builder().launch_default().detach(),
            requirements: RequirementsComponent::builder()
                .launch(GameHardwareRequirements {
                    minimal: HardwareRequirements {
                        cpu: Some(CpuHardwareRequirements {
                            model: LocalizableString::Raw(String::from("Intel Core i5")),
                            cores: Some(4),
                            frequency: Some(5300000000),
                        }),
                        gpu: Some(GpuHardwareRequirements {
                            model: LocalizableString::Raw(String::from("NVIDIA GeForce GT 1030")),
                            vram: Some(2147483648),
                        }),
                        ram: Some(RamHardwareRequirements {
                            size: 8589934592,
                            frequency: Some(1333000000),
                        }),
                        disk: Some(DiskHardwareRequirements {
                            size: 107374182400,
                            disk_type: Some(DiskType::Hdd),
                        }),
                    },
                    optimal: Some(HardwareRequirements {
                        cpu: Some(CpuHardwareRequirements {
                            model: LocalizableString::Raw(String::from("Intel Core i7")),
                            cores: Some(6),
                            frequency: Some(5600000000),
                        }),
                        gpu: Some(GpuHardwareRequirements {
                            model: LocalizableString::Raw(String::from("NVIDIA GeForce GTX 1060")),
                            vram: Some(6442450944),
                        }),
                        ram: Some(RamHardwareRequirements {
                            size: 17179869184,
                            frequency: Some(2400000000),
                        }),
                        disk: Some(DiskHardwareRequirements {
                            size: 161061273600,
                            disk_type: Some(DiskType::Ssd),
                        }),
                    }),
                })
                .detach(),
            maintainers: AsyncFactoryVecDeque::builder().launch_default().detach(),
        };

        let short_buffer = gtk::TextBuffer::new(None);
        let long_buffer = gtk::TextBuffer::new(None);

        short_buffer.set_text(&model.description_short);
        long_buffer.set_text(&model.description_long);

        let widgets = view_output!();

        model.tags.guard().push_back(GameTag::Gambling);
        model.tags.guard().push_back(GameTag::Payments);
        model.tags.guard().push_back(GameTag::AntiCheat);
        model.tags.guard().push_back(GameTag::Workarounds);
        model.tags.guard().push_back(GameTag::GraphicViolence);
        model.tags.guard().push_back(GameTag::PerformanceIssues);
        model.tags.guard().push_back(GameTag::UnsupportedPlatform);

        model
            .maintainers
            .guard()
            .push_back(String::from("Nikita Podvirnyi <krypt0nn@vk.com>"));
        model
            .maintainers
            .guard()
            .push_back(String::from("Maroxy <82662823523516416>"));

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            GamePageAppMsg::Add => {
                println!("Added Game");
            }
        }
    }
}
