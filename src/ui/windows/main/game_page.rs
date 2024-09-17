use adw::prelude::*;
use gtk::prelude::*;

use relm4::{factory::AsyncFactoryVecDeque, prelude::*};

use crate::{
    games::manifest::info::game_tag::GameTag,
    ui::components::{card::CardComponent, game_tags::*},
};

#[derive(Debug)]
pub struct GamePageApp {
    card: AsyncController<CardComponent>,
    title: String,
    developer: String,
    tags: AsyncFactoryVecDeque<GameTagFactory>,
}

#[derive(Debug)]
pub enum GamePageAppMsg {
    Add,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GamePageApp {
    type Init = ();
    type Input = GamePageAppMsg;
    type Output = ();

    view! {
        #[root]
        adw::PreferencesPage {
            adw::PreferencesGroup {
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,
                    set_expand: true,
                    set_spacing: 32,
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 16,
                        model.card.widget(),
                        gtk::Button {
                            set_label: "Add",
                            set_css_classes: &["suggested-action", "pill"],
                            set_halign: gtk::Align::Center,
                            connect_clicked => GamePageAppMsg::Add,
                        }
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 8,
                        gtk::Label {
                            set_markup: &model.title,
                            set_css_classes: &["title-1"],
                        },
                        gtk::Label {
                            set_text: &model.developer,
                            set_css_classes: &["dim-label"],
                        },
                        model.tags.widget() {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_row_spacing: 1,
                            set_column_spacing: 1,
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
        let mut model = Self {
            card: CardComponent::builder()
                .launch(CardComponent {
                    image: Some(String::from("card.jpg")),
                    ..Default::default()
                })
                .detach(),
            title: String::from("Genshin Impact"),
            developer: String::from("MiHoYo"),
            tags: AsyncFactoryVecDeque::builder().launch_default().detach(),
        };
        let widgets = view_output!();

        model.tags.guard().push_back(GameTag::Violence);
        model.tags.guard().push_back(GameTag::Gambling);
        model.tags.guard().push_back(GameTag::Payments);
        model.tags.guard().push_back(GameTag::AntiCheat);
        model.tags.guard().push_back(GameTag::PerformanceIssues);
        model.tags.guard().push_back(GameTag::CompatibilityLayer);
        model.tags.guard().push_back(GameTag::UnsupportedPlatform);

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
