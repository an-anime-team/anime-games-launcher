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
            set_hexpand: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Overlay {
                    #[wrap(Some)]
                    set_child = &adw::Clamp {
                        set_height_request: 340,
                        set_tightening_threshold: 600,

                        model.background.widget() {
                            add_css_class: "card"
                        }
                    },

                    add_overlay = &gtk::Box {
                        set_margin_top: 234,
                        set_margin_start: 16,
                        set_margin_end: 16,
                        set_spacing: 16,

                        model.card.widget(),

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::End,
                            set_margin_top: 126,
                            set_margin_start: 16,

                            gtk::Label {
                                set_halign: gtk::Align::Start,

                                add_css_class: "title-1",

                                #[watch]
                                set_label: &model.title
                            },

                            gtk::Label {
                                set_halign: gtk::Align::Start,

                                #[watch]
                                set_label: &model.developer
                            },

                            gtk::Label {
                                set_halign: gtk::Align::Start,

                                #[watch]
                                set_label: &model.publisher
                            },

                            gtk::Box {
                                set_margin_top: 16,

                                set_hexpand: true,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    gtk::Label {
                                        set_halign: gtk::Align::Start,

                                        set_label: "Played for 317 hours"
                                    },

                                    gtk::Label {
                                        set_halign: gtk::Align::Start,

                                        set_label: "Last played yesterday"
                                    }
                                },

                                gtk::Button {
                                    set_hexpand: true,

                                    set_halign: gtk::Align::End,

                                    add_css_class: "pill",
                                    add_css_class: "suggested-action",

                                    set_label: "Play"
                                }
                            }
                        }
                    }
                },

                gtk::Box {
                    set_margin_top: 130,
                    set_margin_start: 16,
                    set_margin_end: 16,
                    set_spacing: 8,

                    gtk::DropDown {
                        set_width_request: CardSize::Medium.width(),

                        add_css_class: "flat",

                        set_model: Some(&gtk::StringList::new(&[
                            "Global",
                            "China"
                        ]))
                    }
                },

                gtk::Box {
                    set_margin_top: 32,
                    set_spacing: 16,

                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Button {
                        set_label: "123"
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
            GameLibraryDetailsMsg::SetGameInfo { manifest, listener } => {
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
                self.background.emit(LazyPictureComponentMsg::SetImage(Some(ImagePath::lazy_load(&manifest.game.images.background))));
            }
        }
    }
}
