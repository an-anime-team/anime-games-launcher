// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::path::PathBuf;

use adw::prelude::*;
use relm4::prelude::*;

use agl_core::network::downloader::{Downloader, DownloadOptions};

use crate::consts::APP_RESOURCE_PREFIX;
use crate::cache;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImagePath {
    /// Direct path to the existing image.
    Path(PathBuf),

    /// Path to the GTK resource.
    Resource(String),

    /// Lazily load image from the given URL.
    LazyLoad(String)
}

impl ImagePath {
    /// Create new image from the filesystem path.
    ///
    /// ```
    /// ImagePath::path("/tmp/image.png")
    /// ```
    #[inline]
    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }

    /// Create new image stored in the GTK resources.
    /// This function will automatically append the app prefix.
    ///
    /// ```
    /// // APP_RESOURCE_PREFIX/images/icon.png
    /// ImagePath::lazy_load("images/icon.png")
    /// ```
    #[inline]
    pub fn resource(path: impl AsRef<str>) -> Self {
        Self::Resource(format!("{APP_RESOURCE_PREFIX}/{}", path.as_ref()))
    }

    /// Create new lazy loaded image.
    ///
    /// ```
    /// ImagePath::lazy_load("https://example.com/image.png")
    /// ```
    #[inline]
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

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LazyPictureComponent {
    /// Path to the image.
    pub image: Option<ImagePath>,

    /// Picture width.
    pub width: Option<i32>,

    /// Picture height.
    pub height: Option<i32>,

    /// Whether the image should be blurred.
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
                    let cache_path = cache::get_path(url);

                    if cache::is_expired(url, cache::DEFAULT_TTL).unwrap_or(true) {
                        let downloader = Downloader::default();

                        let sender = sender.input_sender().clone();
                        let cache_path = cache_path.clone();

                        downloader.download_with_options(
                            url,
                            cache_path.clone(),
                            DownloadOptions {
                                continue_download: false,
                                on_update: None,
                                on_finish: Some(Box::new(move |_| {
                                    sender.emit(LazyPictureComponentMsg::SetImage(
                                        Some(ImagePath::path(cache_path))
                                    ));
                                }))
                            }
                        );
                    }

                    else {
                        sender.input(LazyPictureComponentMsg::SetImage(
                            Some(ImagePath::path(&cache_path))
                        ));
                    }

                    Some(Some(cache_path))
                },

                _ => None
            },

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

    async fn init(
        model: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>
    ) -> AsyncComponentParts<Self> {
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>
    ) {
        match msg {
            LazyPictureComponentMsg::SetImage(image) => self.image = image,

            LazyPictureComponentMsg::SetWidth(width) => self.width = width,
            LazyPictureComponentMsg::SetHeight(height) => self.height = height,

            LazyPictureComponentMsg::SetBlurred(blurred) => self.blurred = blurred
        }
    }
}
