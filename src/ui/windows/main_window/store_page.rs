use gtk::prelude::*;

use relm4::prelude::*;
use relm4::factory::AsyncFactoryVecDeque;

use crate::ui::components::*;

use crate::games::{
    manifest::info::{
        game_tag::GameTag,
        hardware_requirements::{
            cpu::CpuHardwareRequirements, disk::DiskHardwareRequirements, disk_type::DiskType,
            gpu::GpuHardwareRequirements, ram::RamHardwareRequirements,
            requirements::HardwareRequirements, GameHardwareRequirements,
        },
    },
    prelude::LocalizableString,
};

use super::game_page::*;

#[derive(Debug)]
pub enum StorePageAppMsg {
    Activate,
    ToggleSearching,
    HideGamePage,
    Clicked(DynamicIndex)
}

#[derive(Debug)]
pub enum StorePageAppOutput {
    SetShowBack(bool)
}

#[derive(Debug)]
pub struct StorePageApp {
    games_cards: AsyncFactoryVecDeque<CardsGrid>,
    game_page: AsyncController<GamePageApp>,
    searching: bool,
    show_game_page: bool
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for StorePageApp {
    type Init = ();
    type Input = StorePageAppMsg;
    type Output = StorePageAppOutput;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            #[transition(SlideLeftRight)]
            append = if !model.show_game_page {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_all: 16,
                    set_spacing: 16,

                    gtk::SearchEntry {
                        #[watch]
                        set_visible: model.searching,
                    },

                    adw::ClampScrollable {
                        set_maximum_size: 900,

                        gtk::ScrolledWindow {
                            model.games_cards.widget() {
                                set_row_spacing: 8,
                                set_column_spacing: 8,

                                set_vexpand: true,

                                set_selection_mode: gtk::SelectionMode::None
                            }
                        }
                    }
                }
            } else {
                gtk::Box {
                    model.game_page.widget(),
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self {
            games_cards: AsyncFactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    CardsGridOutput::Clicked(index) => StorePageAppMsg::Clicked(index)
                }),

            game_page: GamePageApp::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    GamePageAppOutput::Hide => StorePageAppMsg::HideGamePage
                }),

            searching: false,
            show_game_page: false
        };

        let widgets = view_output!();

        let mut guard = model.games_cards.guard();

        for i in 0..100 {
            guard.push_back(CardComponent {
                title: Some(format!("Example Game {i}")),
                clickable: true,

                ..CardComponent::medium()
            });
        }

        drop(guard);

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            StorePageAppMsg::ToggleSearching => {
                self.searching = !self.searching;
            }

            StorePageAppMsg::HideGamePage => {
                self.show_game_page = false;
            }

            StorePageAppMsg::Clicked(index) => {
                // Test data
                self.game_page.sender().send(GamePageAppMsg::Update(GamePageAppInit {
                    card_image: String::from("cover.jpg"),
                    carousel_images: vec![String::from("1.jpg"), String::from("2.jpg"), String::from("3.png")],
                    title: String::from("Genshin Impact"),
                    developer: String::from("MiHoYo"),
                    description: String::from("Step into Teyvat, a vast world teeming with life and flowing with elemental energy.

You and your sibling arrived here from another world. Separated by an unknown god, stripped of your powers, and cast into a deep slumber, you now awake to a world very different from when you first arrived.
Thus begins your journey across Teyvat to seek answers from The Seven — the gods of each element. Along the way, prepare to explore every inch of this wondrous world, join forces with a diverse range of characters, and unravel the countless mysteries that Teyvat holds...
MASSIVE OPEN WORLD
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
                    description_split: 565,
                    requirements: GameHardwareRequirements {
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
                    },
                    tags: vec![
                        GameTag::Gambling,
                        GameTag::Payments,
                        GameTag::AntiCheat,
                        GameTag::UnsupportedPlatform,
                    ],
                    maintainers: vec![String::from("Nikita Podvirnyi <krypt0nn@vk.com>"), String::from("Maroxy <82662823523516416>")],
                    repo_name: String::from("an-anime-team"),
                    version: String::from("69.42.0"),
                })).unwrap();

                println!(
                    "Clicked element {}",
                    index.current_index()
                );

                self.show_game_page = true;
            }
            StorePageAppMsg::Activate => {}
        }

        // Update back button visibility
        sender
            .output(StorePageAppOutput::SetShowBack(self.show_game_page))
            .unwrap();
    }
}
