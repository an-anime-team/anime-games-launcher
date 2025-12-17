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

use adw::prelude::*;
use relm4::prelude::*;

use super::lazy_picture::{
    LazyPictureComponent, LazyPictureComponentMsg, ImagePath
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardSize {
    #[default]
    Large,

    Medium,
    Small
}

impl CardSize {
    pub const fn width(&self) -> i32 {
        match self {
            Self::Large  => 240,
            Self::Medium => 160,
            Self::Small  => 40
        }
    }

    pub const fn height(&self) -> i32 {
        // 10:14
        match self {
            Self::Large  => 336,
            Self::Medium => 224,
            Self::Small  => 56
        }
    }

    pub const fn size(&self) -> (i32, i32) {
        (self.width(), self.height())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CardComponentInput {
    SetImage(Option<ImagePath>),
    SetTitle(Option<String>),

    SetSize(CardSize),
    SetClickable(bool),
    SetBlurred(bool),

    EmitClick
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardComponentOutput {
    Clicked
}

#[derive(Debug)]
pub struct CardComponent {
    picture: AsyncController<LazyPictureComponent>,

    /// Size of the card.
    pub size: CardSize,

    /// Title of the card.
    pub title: Option<String>,

    /// Whether the card is clickable.
    pub clickable: bool
}

impl CardComponent {
    pub fn large() -> Self {
        let size = CardSize::Large;

        Self {
            picture: LazyPictureComponent::builder()
                .launch(LazyPictureComponent {
                    image: None,

                    width: Some(size.width()),
                    height: Some(size.height()),

                    blurred: false
                })
                .detach(),

            size,
            title: None,
            clickable: false
        }
    }

    pub fn medium() -> Self {
        let size = CardSize::Medium;

        Self {
            picture: LazyPictureComponent::builder()
                .launch(LazyPictureComponent {
                    image: None,

                    width: Some(size.width()),
                    height: Some(size.height()),

                    blurred: false
                })
                .detach(),

            size,
            title: None,
            clickable: false
        }
    }

    pub fn small() -> Self {
        let size = CardSize::Small;

        Self {
            picture: LazyPictureComponent::builder()
                .launch(LazyPictureComponent {
                    image: None,

                    width: Some(size.width()),
                    height: Some(size.height()),

                    blurred: false
                })
                .detach(),

            size,
            title: None,
            clickable: false
        }
    }

    pub fn with_image(self, image: ImagePath) -> Self {
        self.picture.emit(LazyPictureComponentMsg::SetImage(Some(image)));

        self
    }

    pub fn with_title(mut self, title: impl ToString) -> Self {
        self.title = Some(title.to_string());

        self
    }

    #[inline]
    pub const fn with_clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;

        self
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for CardComponent {
    type Init = Self;
    type Input = CardComponentInput;
    type Output = CardComponentOutput;

    view! {
        #[root]
        adw::Clamp {
            #[watch]
            set_maximum_size: model.size.width(),

            gtk::Box {
                // set_vexpand: true,
                // set_hexpand: true,

                set_orientation: gtk::Orientation::Vertical,

                gtk::Overlay {
                    #[watch]
                    set_tooltip?: &model.title,

                    model.picture.widget() {
                        add_css_class: "card"
                    },

                    add_overlay = &gtk::Button {
                        add_css_class: "flat",

                        #[watch]
                        set_visible: model.clickable,

                        connect_clicked => CardComponentInput::EmitClick
                    }
                },

                gtk::Label {
                    set_margin_top: 12,

                    set_halign: gtk::Align::Center,
                    set_justify: gtk::Justification::Center,

                    #[watch]
                    set_visible: model.title.is_some(),

                    #[watch]
                    set_label?: model.title.clone()
                        .map(|title| {
                            let max_chars = model.size.width() as usize / 8;

                            if title.chars().count() <= max_chars {
                                return title;
                            }

                            title.chars()
                                .take(max_chars.checked_sub(3).unwrap_or_default())
                                .chain("...".chars())
                                .collect::<String>()
                        })
                        .as_deref()
                }
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
        sender: AsyncComponentSender<Self>
    ) {
        match msg {
            CardComponentInput::SetTitle(title) => self.title = title,

            CardComponentInput::SetImage(image) => {
                self.picture.emit(LazyPictureComponentMsg::SetImage(image));
            }

            CardComponentInput::SetSize(size) => {
                self.size = size;

                self.picture.emit(LazyPictureComponentMsg::SetWidth(Some(size.width())));
                self.picture.emit(LazyPictureComponentMsg::SetHeight(Some(size.height())));
            }

            CardComponentInput::SetClickable(clickable) => self.clickable = clickable,

            CardComponentInput::SetBlurred(blurred) => {
                self.picture.emit(LazyPictureComponentMsg::SetBlurred(blurred));
            }

            CardComponentInput::EmitClick => {
                let _ = sender.output(CardComponentOutput::Clicked);
            }
        }
    }
}
