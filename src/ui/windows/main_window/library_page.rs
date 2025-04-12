use std::sync::Arc;

use relm4::prelude::*;
use adw::prelude::*;

use tokio::sync::mpsc::UnboundedSender;
use unic_langid::LanguageIdentifier;

use super::*;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum LibraryPageInput {
    SetGeneration(GenerationManifest),

    AddGameFromGeneration {
        url: String,
        manifest: GameManifest,
        listener: UnboundedSender<SyncGameCommand>
    },

    Activate,

    GameRowSelected(usize),
    HideOtherGamesEditions(DynamicIndex),

    ShowGameDetails {
        game: DynamicIndex,
        variant: Option<DynamicIndex>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryPageOutput {
    SetShowBack(bool)
}

pub struct LibraryPage {
    cards_list: AsyncFactoryVecDeque<CardsList>,
    game_details: AsyncController<GameLibraryDetails>,

    download_manager: AsyncController<DownloadManagerWindow>,

    #[allow(clippy::type_complexity)]
    games: Vec<(String, Arc<GameManifest>, Vec<GameEdition>, UnboundedSender<SyncGameCommand>)>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for LibraryPage {
    type Init = adw::ApplicationWindow;
    type Input = LibraryPageInput;
    type Output = LibraryPageOutput;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            adw::NavigationSplitView {
                set_vexpand: true,
                set_hexpand: true,

                #[wrap(Some)]
                set_sidebar = &adw::NavigationPage {
                    // Supress Adwaita-WARNING **: AdwNavigationPage is missing a title
                    set_title: "Games",

                    #[wrap(Some)]
                    set_child = model.cards_list.widget() {
                        add_css_class: "navigation-sidebar",

                        connect_row_activated[sender] => move |_, row| {
                            sender.input(LibraryPageInput::GameRowSelected(row.index() as usize));
                        }
                    }
                },

                #[wrap(Some)]
                set_content = &adw::NavigationPage {
                    // Supress Adwaita-WARNING **: AdwNavigationPage is missing a title
                    set_title: "Details",

                    set_hexpand: true,

                    #[wrap(Some)]
                    set_child = model.game_details.widget(),
                }
            }
        }
    }

    async fn init(parent: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            cards_list: AsyncFactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    CardsListOutput::Selected { card: game, variant }
                        => LibraryPageInput::ShowGameDetails { game, variant },

                    CardsListOutput::HideOtherVariants(index)
                        => LibraryPageInput::HideOtherGamesEditions(index)
                }),

            game_details: GameLibraryDetails::builder()
                .launch(parent)
                .detach(),

            download_manager: DownloadManagerWindow::builder()
                .launch(())
                .detach(),

            games: Vec::new()
        };

        model.cards_list.widget().connect_row_selected(|_, row| {
            if let Some(row) = row {
                row.emit_activate();
            }
        });

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            LibraryPageInput::SetGeneration(generation) => {
                self.games.clear();
                self.cards_list.guard().clear();

                let download_manager = self.download_manager.sender().to_owned();

                std::thread::spawn(move || {
                    if let Err(err) = serve_generation(sender, download_manager, generation) {
                        tracing::error!(?err, "Failed to serve generation");
                    }
                });
            }

            LibraryPageInput::AddGameFromGeneration { url, manifest, listener } => {
                let config = config::get();

                let lang = config.general.language.parse::<LanguageIdentifier>();

                let (send, recv) = tokio::sync::oneshot::channel();

                // TODO: better errors handling
                if let Err(err) = listener.send(SyncGameCommand::GetEditions { listener: send }) {
                    tracing::error!(?err, "Failed to request game's editions");

                    return;
                }

                // TODO: build Arc-s here
                let editions = match recv.await {
                    Ok(Ok(editions)) => editions,

                    Ok(Err(err)) => {
                        tracing::error!(?err, "Failed to request game's editions");

                        return;
                    }

                    Err(err) => {
                        tracing::error!(?err, "Failed to request game's editions");

                        return;
                    }
                };

                self.cards_list.guard().push_back(CardsListInit {
                    image: ImagePath::LazyLoad(manifest.game.images.poster.clone()),

                    title: match &lang {
                        Ok(lang) => manifest.game.title.translate(lang).to_string(),
                        Err(_) => manifest.game.title.default_translation().to_string()
                    },

                    variants: Some(editions.iter()
                        .map(|edition| {
                            match &lang {
                                Ok(lang) => edition.title.translate(lang).to_string(),
                                Err(_) => edition.title.default_translation().to_string()
                            }
                        })
                        .collect::<Vec<_>>())
                });

                self.games.push((url, Arc::new(manifest), editions, listener));
            }

            LibraryPageInput::GameRowSelected(index) => {
                self.cards_list.send(index, CardsListInput::EmitClick);
            }

            LibraryPageInput::HideOtherGamesEditions(index) => {
                self.cards_list.broadcast(CardsListInput::HideVariantsExcept(index));
            }

            LibraryPageInput::ShowGameDetails { game, variant } => {
                // FIXME: don't update details page if it's already open for the given game.

                self.cards_list.broadcast(CardsListInput::HideVariantsExcept(game.clone()));

                // TODO: proper errors handling
                let Some((_, manifest, editions, listener)) = self.games.get(game.current_index()) else {
                    tracing::error!(
                        game = game.current_index(),
                        variant = variant.map(|variant| variant.current_index()),
                        "Failed to read game info"
                    );

                    return;
                };

                let edition = match &variant {
                    Some(variant) => editions.get(variant.current_index()),
                    None => editions.first()
                };

                let Some(edition) = edition.cloned() else {
                    tracing::error!(
                        game = game.current_index(),
                        variant = variant.map(|variant| variant.current_index()),
                        "Failed to get game edition"
                    );

                    return;
                };

                self.game_details.emit(GameLibraryDetailsMsg::SetGameInfo {
                    manifest: manifest.to_owned(),
                    edition,
                    listener: listener.clone()
                });
            }

            LibraryPageInput::Activate => {
                // Update back button visibility when switching pages
            }
        }
    }
}
