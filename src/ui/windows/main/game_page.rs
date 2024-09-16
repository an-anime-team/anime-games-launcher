use adw::prelude::*;
use gtk::prelude::*;

use relm4::prelude::*;

use crate::ui::components::card::CardComponent;

#[derive(Debug)]
pub struct GamePageApp {
    card: AsyncController<CardComponent>,
    title: String,
    developer: String,
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
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 16,
                            gtk::Button {
                                set_label: "Add",
                                set_css_classes: &["suggested-action", "pill"],
                                set_halign: gtk::Align::Center,
                                connect_clicked => GamePageAppMsg::Add,
                            }
                        }
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 16,
                        gtk::Label {
                            set_markup: &format!("<big><b>{}</b></big>", model.title),
                        },
                        gtk::Label {
                            set_text: &model.developer,
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
                    image: Some(String::from("card.jpg")),
                    ..Default::default()
                })
                .detach(),
            title: String::from("Genshin Impact"),
            developer: String::from("MiHoYo"),
        };
        let widgets = view_output!();

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
