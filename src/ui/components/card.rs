use std::path::PathBuf;

use adw::prelude::*;
use relm4::prelude::*;

use crate::prelude::*;

// 10:14
pub const DEFAULT_SIZE: (i32, i32) = (240, 336);
pub const MEDIUM_SIZE: (i32, i32) = (160, 224);
pub const SMALL_SIZE: (i32, i32) = (40, 56);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CardImage {
    /// Direct path to the existing image.
    Path(PathBuf),

    /// Path to the GTK resource.
    Resource(String),

    /// Lazily load the image from the given URL.
    LazyLoad(String)
}

impl CardImage {
    #[inline]
    /// Create new image from the filesystem path.
    /// 
    /// ```
    /// CardImage::lazy_load("/tmp/image.png")
    /// ```
    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }

    #[inline]
    /// Create new image stored in the GTK resources.
    /// This function will automatically append the app prefix.
    /// 
    /// ```
    /// // APP_RESOURCE_PREFIX/images/icon.png
    /// CardImage::lazy_load("images/icon.png")
    /// ```
    pub fn resource(path: impl AsRef<str>) -> Self {
        Self::Resource(format!("{APP_RESOURCE_PREFIX}/{}", path.as_ref()))
    }

    #[inline]
    /// Create new lazy loaded image.
    /// 
    /// ```
    /// CardImage::lazy_load("https://example.com/image.png")
    /// ```
    pub fn lazy_load(url: impl ToString) -> Self {
        Self::LazyLoad(url.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CardComponentInput {
    SetImage(Option<CardImage>),
    SetTitle(Option<String>),

    SetWidth(i32),
    SetHeight(i32),

    SetClickable(bool),
    SetBlurred(bool),

    EmitClick
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardComponentOutput {
    Clicked
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardComponent {
    pub image: Option<CardImage>,
    pub title: Option<String>,

    pub width: i32,
    pub height: i32,

    pub clickable: bool,
    pub blurred: bool
}

impl Default for CardComponent {
    #[inline]
    fn default() -> Self {
        Self {
            image: None,
            title: None,

            width: DEFAULT_SIZE.0,
            height: DEFAULT_SIZE.1,

            clickable: false,
            blurred: false
        }
    }
}

impl CardComponent {
    #[inline]
    pub fn medium() -> Self {
        Self {
            width: MEDIUM_SIZE.0,
            height: MEDIUM_SIZE.1,

            ..Self::default()
        }
    }

    #[inline]
    pub fn small() -> Self {
        Self {
            width: SMALL_SIZE.0,
            height: SMALL_SIZE.1,

            ..Self::default()
        }
    }

    #[inline]
    pub fn with_image(mut self, image: CardImage) -> Self {
        self.image = Some(image);

        self
    }

    #[inline]
    pub fn with_title(mut self, title: impl ToString) -> Self {
        self.title = Some(title.to_string());

        self
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for CardComponent {
    type Init = CardComponent;
    type Input = CardComponentInput;
    type Output = CardComponentOutput;

    view! {
        #[root]
        adw::Clamp {
            #[watch]
            set_maximum_size: model.width,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Overlay {
                    #[watch]
                    set_tooltip?: &model.title,

                    gtk::Picture {
                        set_valign: gtk::Align::Start,
                        set_halign: gtk::Align::Start,

                        set_content_fit: gtk::ContentFit::Cover,

                        add_css_class: "card",

                        #[watch]
                        set_size_request: (model.width, model.height),

                        #[watch]
                        set_opacity: if model.blurred { 0.4 } else { 1.0 },

                        #[watch]
                        set_filename?: match &model.image {
                            Some(CardImage::Path(path)) => Some(Some(path.to_path_buf())),

                            Some(CardImage::LazyLoad(url)) => {
                                let sender = sender.input_sender();

                                let (path_sender, path_reader) = tokio::sync::oneshot::channel();

                                {
                                    let sender = sender.clone();

                                    tokio::spawn(async move {
                                        if let Ok(path) = path_reader.await {
                                            sender.emit(CardComponentInput::SetImage(Some(CardImage::path(path))));
                                        }
                                    });
                                }

                                let path = FileCache::default().swap(url, path_sender);

                                if let Some(path) = &path {
                                    sender.emit(CardComponentInput::SetImage(Some(CardImage::path(path))));
                                }

                                Some(path)
                            },

                            _ => None
                        },

                        // FUCK YOU, GTK-RS !!!

                        #[watch]
                        set_resource?: if let Some(CardImage::Resource(path)) = &model.image {
                            Some(Some(path.as_str()))
                        } else {
                            None
                        },

                        #[watch]
                        set_resource?: if let Some(CardImage::LazyLoad(_)) = &model.image {
                            // let path = format!("{APP_RESOURCE_PREFIX}/images/missing-card.png");

                            Some(Some("/moe/launcher/anime-games-launcher/images/missing-card.png"))
                        } else {
                            None
                        }
                    },

                    add_overlay = &gtk::Button {
                        add_css_class: "flat",

                        #[watch]
                        set_visible: model.clickable,

                        connect_clicked => CardComponentInput::EmitClick
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,

                    set_margin_all: 12,

                    #[watch]
                    set_visible: model.title.is_some(),

                    gtk::Label {
                        #[watch]
                        set_label?: &model.title
                    }
                }
            }
        }
    }

    #[inline]
    async fn init(model: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            CardComponentInput::SetTitle(title) => self.title = title,
            CardComponentInput::SetImage(image) => self.image = image,

            CardComponentInput::SetWidth(width) => self.width = width,
            CardComponentInput::SetHeight(height) => self.height = height,

            CardComponentInput::SetClickable(clickable) => self.clickable = clickable,
            CardComponentInput::SetBlurred(blurred) => self.blurred = blurred,

            CardComponentInput::EmitClick => {
                let _ = sender.output(CardComponentOutput::Clicked);
            }
        }
    }
}
