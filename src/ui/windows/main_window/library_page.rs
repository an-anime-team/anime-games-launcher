use std::sync::Arc;

use relm4::prelude::*;
use adw::prelude::*;

use tokio::sync::mpsc::UnboundedSender;
use unic_langid::LanguageIdentifier;

use super::*;

#[derive(Debug, Clone)]
struct GameInfo {
    pub manifest: Arc<GameManifest>,
    pub editions: Option<Vec<GameEdition>>,
    pub listener: UnboundedSender<SyncGameCommand>
}

#[derive(Debug)]
pub enum LibraryPageInput {
    SpawnLuauEngine {
        generation: GenerationManifest,
        validator: AuthorityValidator
    },

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
    },

    ShowToast {
        message: LocalizableString,
        action: Option<(LocalizableString, mlua::Function)>
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

    main_window: Option<adw::ApplicationWindow>,
    toast_overlay: Option<adw::ToastOverlay>,

    games: Vec<GameInfo>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for LibraryPage {
    type Init = adw::ApplicationWindow;
    type Input = LibraryPageInput;
    type Output = LibraryPageOutput;

    view! {
        #[root]
        toast_overlay = adw::ToastOverlay {
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
        let mut model = Self {
            cards_list: AsyncFactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), |msg| match msg {
                    CardsListOutput::Selected { card: game, variant }
                        => LibraryPageInput::ShowGameDetails { game, variant },

                    CardsListOutput::HideOtherVariants(index)
                        => LibraryPageInput::HideOtherGamesEditions(index)
                }),

            game_details: GameLibraryDetails::builder()
                .launch(parent.clone())
                .detach(),

            download_manager: DownloadManagerWindow::builder()
                .launch(())
                .detach(),

            main_window: None,
            toast_overlay: None,

            games: Vec::new()
        };

        model.cards_list.widget().connect_row_selected(|_, row| {
            if let Some(row) = row {
                row.emit_activate();
            }
        });

        let widgets = view_output!();

        model.main_window = Some(parent);
        model.toast_overlay = Some(widgets.toast_overlay.clone());

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            LibraryPageInput::SpawnLuauEngine { generation, validator } => {
                self.games.clear();
                self.cards_list.guard().clear();

                let download_manager = self.download_manager.sender().to_owned();

                // TODO: we don't do this now, but in future this event could be called
                //       multiple times, so we would need to kill unused threads.
                std::thread::spawn(move || {
                    if let Err(err) = serve_generation(sender, download_manager, generation, validator) {
                        tracing::error!(?err, "Failed to serve generation");
                    }
                });
            }

            LibraryPageInput::AddGameFromGeneration { url: _, manifest, listener } => {
                let config = config::get();

                let lang = config.general.language.parse::<LanguageIdentifier>();

                let (send, recv) = tokio::sync::oneshot::channel();

                // TODO: better errors handling
                if let Err(err) = listener.send(SyncGameCommand::GetEditions { listener: send }) {
                    tracing::error!(?err, "Failed to request game editions");

                    return;
                }

                // TODO: build Arc-s here
                let editions = match recv.await {
                    Ok(Ok(editions)) => editions,

                    Ok(Err(err)) => {
                        tracing::error!(?err, "Failed to request game editions");

                        return;
                    }

                    Err(err) => {
                        tracing::error!(?err, "Failed to request game editions");

                        return;
                    }
                };

                self.cards_list.guard().push_back(CardsListInit {
                    image: ImagePath::LazyLoad(manifest.game.images.poster.clone()),

                    title: match &lang {
                        Ok(lang) => manifest.game.title.translate(lang).to_string(),
                        Err(_) => manifest.game.title.default_translation().to_string()
                    },

                    variants: editions.as_ref().map(|editions| {
                        editions.iter()
                            .map(|edition| {
                                match &lang {
                                    Ok(lang) => edition.title.translate(lang).to_string(),
                                    Err(_) => edition.title.default_translation().to_string()
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                });

                self.games.push(GameInfo {
                    manifest: Arc::new(manifest),
                    editions,
                    listener
                });
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
                let Some(game) = self.games.get(game.current_index()) else {
                    tracing::error!(
                        game = game.current_index(),
                        variant = variant.map(|variant| variant.current_index()),
                        "Failed to read game info"
                    );

                    return;
                };

                let edition = match (&variant, &game.editions) {
                    (_, None) => None,
                    (Some(variant), Some(editions)) => editions.get(variant.current_index()),
                    (None, Some(editions)) => editions.first()
                };

                self.game_details.emit(GameLibraryDetailsMsg::UpdateGameMetadata {
                    manifest: game.manifest.clone(),
                    listener: game.listener.clone(),
                    edition: edition.cloned()
                });
            }

            LibraryPageInput::Activate => {
                // Update back button visibility when switching pages
            }

            LibraryPageInput::ShowToast { message, action } => {
                if let Some(toast_overlay) = self.toast_overlay.as_ref() {
                    let config = config::get();

                    let language = config.general.language.parse::<LanguageIdentifier>();

                    let message = match &language {
                        Ok(language) => message.translate(language),
                        Err(_) => message.default_translation()
                    };

                    let toast = adw::Toast::new(message);

                    if let Some((label, callback)) = action {
                        let label = match &language {
                            Ok(language) => label.translate(language),
                            Err(_) => label.default_translation()
                        };

                        toast.set_button_label(Some(label));

                        toast.connect_button_clicked(move |_| {
                            if let Err(err) = callback.call::<()>(()) {
                                tracing::error!(?err, "Failed to execute lua engine callback to the toast button");
                            }
                        });
                    }

                    toast_overlay.add_toast(toast);
                }
            }
        }
    }
}
