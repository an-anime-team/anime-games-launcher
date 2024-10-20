use std::sync::Arc;

use adw::prelude::*;
use relm4::prelude::*;

use tokio::sync::mpsc::UnboundedSender;

use unic_langid::LanguageIdentifier;

use crate::prelude::*;

use super::*;

#[derive(Debug)]
pub enum GameLibraryDetailsMsg {
    SetGameInfo {
        manifest: Arc<GameManifest>,
        edition: GameEdition,
        listener: UnboundedSender<SyncGameCommand>
    }
}

#[derive(Debug)]
pub struct GameLibraryDetails {
    card: AsyncController<CardComponent>,
    background: AsyncController<LazyPictureComponent>,

    listener: Option<UnboundedSender<SyncGameCommand>>,

    title: String,
    developer: String,
    publisher: String
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameLibraryDetails {
    type Init = ();
    type Input = GameLibraryDetailsMsg;
    type Output = ();

    view! {
        adw::Clamp {
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                set_margin_top: 16,
                set_spacing: 16,

                gtk::Label {
                    set_halign: gtk::Align::Start,

                    add_css_class: "title-1",

                    #[watch]
                    set_label: &model.title
                },

                model.background.widget() {
                    add_css_class: "card"
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    set_spacing: 16,

                    gtk::Button {
                        add_css_class: "pill",
                        add_css_class: "suggested-action",

                        set_label: "Play"
                    }
                }
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, _sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            card: CardComponent::builder()
                .launch(CardComponent::medium())
                .detach(),

            background: LazyPictureComponent::builder()
                .launch(LazyPictureComponent::default())
                .detach(),

            listener: None,

            title: String::new(),
            developer: String::new(),
            publisher: String::new()
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            GameLibraryDetailsMsg::SetGameInfo { manifest, edition, listener } => {
                let config = config::get();

                let lang = config.general.language.parse::<LanguageIdentifier>();

                let title = match &lang {
                    Ok(lang) => manifest.game.title.translate(lang),
                    Err(_) => manifest.game.title.default_translation()
                };

                let developer = match &lang {
                    Ok(lang) => manifest.game.developer.translate(lang),
                    Err(_) => manifest.game.developer.default_translation()
                };

                let publisher = match &lang {
                    Ok(lang) => manifest.game.publisher.translate(lang),
                    Err(_) => manifest.game.publisher.default_translation()
                };

                self.listener = Some(listener);

                self.title = title.to_string();
                self.developer = developer.to_string();
                self.publisher = publisher.to_string();

                self.card.emit(CardComponentInput::SetImage(Some(ImagePath::lazy_load(&manifest.game.images.poster))));

                // Little trolling. I think you can sorry me.
                let date = time::OffsetDateTime::now_utc();

                let image = if (date.month() == time::Month::April && date.day() == 1) || (date.hour() == 19 && date.minute() == 17) {
                    tracing::info!("＜( ￣︿￣)");

                    ImagePath::resource("images/april-fools.jpg")
                } else {
                    ImagePath::lazy_load(&manifest.game.images.background)
                };

                self.background.emit(LazyPictureComponentMsg::SetImage(Some(image)));
            }
        }
    }
}
