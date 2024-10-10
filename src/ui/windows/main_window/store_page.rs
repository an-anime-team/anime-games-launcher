use gtk::prelude::*;

use relm4::prelude::*;
use relm4::factory::AsyncFactoryVecDeque;

use unic_langid::LanguageIdentifier;

use crate::ui::components::*;

use super::*;

#[derive(Debug)]
pub enum StorePageInput {
    AddGame {
        url: String,
        manifest: Arc<GameManifest>
    },
    Activate,
    ToggleSearching,
    HideGamePage,
    OpenGameDetails(DynamicIndex)
}

#[derive(Debug)]
pub enum StorePageOutput {
    SetShowBack(bool)
}

#[derive(Debug)]
pub struct StorePage {
    games_cards: AsyncFactoryVecDeque<CardsGrid>,
    game_details_page: AsyncController<GameDetailsPage>,

    games: Vec<Arc<GameManifest>>,

    searching: bool,
    show_game_page: bool
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for StorePage {
    type Init = ();
    type Input = StorePageInput;
    type Output = StorePageOutput;

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
                    model.game_details_page.widget(),
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            games_cards: AsyncFactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    CardsGridOutput::Clicked(index) => StorePageInput::OpenGameDetails(index)
                }),

            game_details_page: GameDetailsPage::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    GameDetailsPageOutput::Hide => StorePageInput::HideGamePage
                }),

            games: Vec::new(),

            searching: false,
            show_game_page: false
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            StorePageInput::AddGame { url: _, manifest } => {
                let config = config::get();

                let lang = config.general.language.parse::<LanguageIdentifier>().ok();

                let title = match &lang {
                    Some(lang) => manifest.game.title.translate(lang),
                    None => manifest.game.title.default_translation()
                };

                self.games_cards.guard().push_back(CardComponent {
                    title: Some(title.to_string()),
                    clickable: true,

                    ..CardComponent::medium()
                });

                self.games.push(manifest);
            }

            StorePageInput::ToggleSearching => {
                self.searching = !self.searching;
            }

            StorePageInput::HideGamePage => {
                self.show_game_page = false;
            }

            StorePageInput::OpenGameDetails(index) => {
                let Some(game) = self.games.get(index.current_index()) else {
                    tracing::error!(
                        index = index.current_index(),
                        length = self.games.len(),
                        "Trying to open details page of an unexisting game"
                    );

                    return;
                };

                self.game_details_page.emit(GameDetailsPageInput::SetGameInfo(game.clone()));

                self.show_game_page = true;
            }

            StorePageInput::Activate => {
                // Update back button visibility when switching pages
            }
        }

        // Update back button visibility
        sender
            .output(StorePageOutput::SetShowBack(self.show_game_page))
            .unwrap();
    }
}
