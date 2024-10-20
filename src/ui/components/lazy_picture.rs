use std::path::PathBuf;

use adw::prelude::*;
use relm4::prelude::*;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImagePath {
    /// Direct path to the existing image.
    Path(PathBuf),

    /// Path to the GTK resource.
    Resource(String),

    /// Lazily load the image from the given URL.
    LazyLoad(String)
}

impl ImagePath {
    #[inline]
    /// Create new image from the filesystem path.
    ///
    /// ```
    /// ImagePath::path("/tmp/image.png")
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
    /// ImagePath::lazy_load("images/icon.png")
    /// ```
    pub fn resource(path: impl AsRef<str>) -> Self {
        Self::Resource(format!("{APP_RESOURCE_PREFIX}/{}", path.as_ref()))
    }

    #[inline]
    /// Create new lazy loaded image.
    ///
    /// ```
    /// ImagePath::lazy_load("https://example.com/image.png")
    /// ```
    pub fn lazy_load(url: impl ToString) -> Self {
        Self::LazyLoad(url.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LazyPictureComponentMsg {
    SetImage(Option<ImagePath>),

    SetWidth(Option<i32>),
    SetHeight(Option<i32>),

    SetBlurred(bool)
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct LazyPictureComponent {
    pub image: Option<ImagePath>,

    pub width: Option<i32>,
    pub height: Option<i32>,

    pub blurred: bool
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for LazyPictureComponent {
    type Init = Self;
    type Input = LazyPictureComponentMsg;
    type Output = ();

    view! {
        #[root]
        gtk::Picture {
            set_valign: gtk::Align::Start,
            set_halign: gtk::Align::Start,

            set_content_fit: gtk::ContentFit::Cover,

            #[watch]
            set_width_request?: model.width,

            #[watch]
            set_height_request?: model.height,

            #[watch]
            set_opacity: if model.blurred { 0.4 } else { 1.0 },

            #[watch]
            set_filename?: match &model.image {
                Some(ImagePath::Path(path)) => Some(Some(path.to_path_buf())),

                Some(ImagePath::LazyLoad(url)) => {
                    let sender = sender.input_sender();

                    let (path_sender, path_reader) = tokio::sync::oneshot::channel();

                    {
                        let sender = sender.clone();

                        tokio::spawn(async move {
                            if let Ok(path) = path_reader.await {
                                sender.emit(LazyPictureComponentMsg::SetImage(Some(ImagePath::path(path))));
                            }
                        });
                    }

                    let path = FileCache::default().swap(url, path_sender);

                    if let Some(path) = &path {
                        sender.emit(LazyPictureComponentMsg::SetImage(Some(ImagePath::path(path))));
                    }

                    Some(path)
                },

                _ => None
            },

            // FUCK YOU, GTK-RS !!!

            #[watch]
            set_resource?: if let Some(ImagePath::Resource(path)) = &model.image {
                Some(Some(path.as_str()))
            } else {
                None
            },

            #[watch]
            set_resource?: if let Some(ImagePath::LazyLoad(_)) = &model.image {
                // let path = format!("{APP_RESOURCE_PREFIX}/images/missing-card.png");

                Some(Some("/moe/launcher/anime-games-launcher/images/missing-card.png"))
            } else {
                None
            }
        }
    }

    #[inline]
    async fn init(model: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            LazyPictureComponentMsg::SetImage(image) => self.image = image,

            LazyPictureComponentMsg::SetWidth(width) => self.width = width,
            LazyPictureComponentMsg::SetHeight(height) => self.height = height,

            LazyPictureComponentMsg::SetBlurred(blurred) => self.blurred = blurred
        }
    }
}
