use gtk::prelude::*;
use adw::prelude::*;

use relm4::prelude::*;
use relm4::factory::*;

use crate::ui::components::game_details::GameDetailsInit;
use crate::ui::components::prelude::*;

#[derive(Debug, Clone)]
pub enum LibraryPageAppMsg {
    ShowGameDetails(DynamicIndex)
}

#[derive(Debug)]
pub struct LibraryPageApp {
    cards_list: AsyncFactoryVecDeque<CardsListFactory>,
    game_details: AsyncController<GameDetails>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for LibraryPageApp {
    type Init = ();
    type Input = LibraryPageAppMsg;
    type Output = ();

    view! {
        #[root]
        adw::NavigationSplitView {
            set_vexpand: true,
            set_hexpand: true,

            #[wrap(Some)]
            set_sidebar = &adw::NavigationPage {
                // Supress Adwaita-WARNING **: AdwNavigationPage is missing a title
                set_title: "Games",

                #[wrap(Some)]
                set_child = model.cards_list.widget(),
            },

            #[wrap(Some)]
            set_content = &adw::NavigationPage {
                set_hexpand: true,

                // Supress Adwaita-WARNING **: AdwNavigationPage is missing a title
                set_title: "Details",

                #[wrap(Some)]
                set_child = model.game_details.widget(),
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let mut model = Self {
            cards_list: AsyncFactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| {
                    match msg {
                        CardsListFactoryOutput::Selected(index)
                            => LibraryPageAppMsg::ShowGameDetails(index)
                    }
                }),

            game_details: GameDetails::builder()
                .launch(())
                .detach()
        };

        model.cards_list.widget().add_css_class("navigation-sidebar");

        model.cards_list.widget().connect_row_selected(|_, row| {
            if let Some(row) = row {
                row.emit_activate();
            }
        });

        model.cards_list.guard().push_back(CardsListFactoryInit::new("Genshin Impact", "/var/home/observer/projects/new-anime-core/anime-games-launcher/assets/images/games/genshin/card.jpg"));
        model.cards_list.guard().push_back(CardsListFactoryInit::new("Honkai Impact 3rd", "/var/home/observer/projects/new-anime-core/anime-games-launcher/assets/images/games/honkai/card.jpg"));
        model.cards_list.guard().push_back(CardsListFactoryInit::new("Honkai: Star Rail", "/var/home/observer/projects/new-anime-core/anime-games-launcher/assets/images/games/star-rail/card.jpg"));
        model.cards_list.guard().push_back(CardsListFactoryInit::new("Punishing: Gray Raven", "/var/home/observer/projects/new-anime-core/anime-games-launcher/assets/images/games/pgr/card.jpg"));

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            LibraryPageAppMsg::ShowGameDetails(index) => {
                self.game_details.emit(GameDetailsInput::Update(GameDetailsInit {
                    title: String::from("Genshin Impact"),
                    card_image: String::from("/var/home/observer/projects/new-anime-core/anime-games-launcher/assets/images/games/genshin/card.jpg"),
                    background_image: String::from("/var/home/observer/projects/new-anime-core/anime-games-launcher/assets/images/games/genshin/background.jpg")
                }));
            }
        }
    }
}
