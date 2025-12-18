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

use super::lazy_picture::ImagePath;
use super::card::CardComponent;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CardsListInit {
    /// Card image path.
    pub image: ImagePath,

    /// Card title.
    pub title: String,

    /// Variants titles.
    pub variants: Option<Box<[String]>>
}

impl CardsListInit {
    pub fn new(
        image: ImagePath,
        title: impl ToString,
        variants: Option<impl IntoIterator<Item = String>>
    ) -> Self {
        Self {
            image,
            title: title.to_string(),
            variants: variants.map(|variants| variants.into_iter().collect())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardsListInput {
    EmitClick,
    ShowVariants,
    HideVariants,
    HideVariantsExcept(DynamicIndex)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardsListOutput {
    Selected {
        card: DynamicIndex,
        variant: Option<DynamicIndex>
    },

    HideOtherVariants(DynamicIndex)
}

#[derive(Debug)]
pub struct CardsList {
    card: AsyncController<CardComponent>,
    variants: AsyncFactoryVecDeque<CardVariantsList>,

    title: String,
    index: DynamicIndex,

    has_variants: bool,
    show_variants: bool
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for CardsList {
    type Init = CardsListInit;
    type Input = CardsListInput;
    type Output = CardsListOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::ListBoxRow {
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                set_spacing: 6,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    set_spacing: 12,

                    self.card.widget() -> &adw::Clamp {
                        set_margin_top: 6,
                        set_margin_bottom: 6
                    },

                    gtk::Label {
                        set_label: &self.title
                    }
                },

                self.variants.widget().clone() -> gtk::ListBox {
                    add_css_class: "navigation-sidebar",

                    set_margin_bottom: 6,

                    #[watch]
                    set_visible: self.show_variants
                }
            },

            set_activatable: true,

            connect_activate => CardsListInput::EmitClick
        }
    }

    async fn init_model(
        init: Self::Init,
        index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>
    ) -> Self {
        let mut model = Self {
            card: CardComponent::builder()
                .launch(CardComponent::small().with_image(init.image))
                .detach(),

            variants: AsyncFactoryVecDeque::builder()
                .launch_default()
                .detach(),

            title: init.title,

            index: index.to_owned(),

            has_variants: false,
            show_variants: false
        };

        if let Some(variants) = init.variants {
            let mut guard = model.variants.guard();

            for variant in variants {
                guard.push_back(variant);
            }

            model.has_variants = true;
        }

        model
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncFactorySender<Self>
    ) {
        match msg {
            CardsListInput::EmitClick => {
                let _ = sender.output(CardsListOutput::HideOtherVariants(self.index.clone()));

                if self.has_variants {
                    #[allow(clippy::collapsible_if)]
                    if self.variants.widget().selected_row().is_none() {
                        if let Some(variant) = self.variants.widget().first_child() {
                            self.variants.widget().select_row(Some(&unsafe {
                                variant.unsafe_cast::<gtk::ListBoxRow>()
                            }));
                        }
                    }

                    let variant = self.variants.widget()
                        .selected_row()
                        .and_then(|row| self.variants.get(row.index() as usize))
                        .map(|variant| variant.index.clone());

                    let _ = sender.output(CardsListOutput::Selected {
                        card: self.index.clone(),
                        variant
                    });

                    self.show_variants = true;
                }

                else {
                    let _ = sender.output(CardsListOutput::Selected {
                        card: self.index.clone(),
                        variant: None
                    });

                    self.show_variants = false;
                }
            }

            CardsListInput::ShowVariants => self.show_variants = true,
            CardsListInput::HideVariants => self.show_variants = false,

            CardsListInput::HideVariantsExcept(index) => {
                if self.index != index {
                    self.show_variants = false;
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardVariantsList {
    title: String,
    index: DynamicIndex
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for CardVariantsList {
    type Init = String;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::ListBoxRow {
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 12,

                gtk::Image {
                    set_icon_name: Some("globe-symbolic")
                },

                gtk::Label {
                    set_halign: gtk::Align::Start,

                    set_margin_top: 6,
                    set_margin_bottom: 6,

                    #[watch]
                    set_label: &self.title,
                }
            }
        }
    }

    async fn init_model(
        title: Self::Init,
        index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>
    ) -> Self {
        Self {
            title,
            index: index.to_owned()
        }
    }
}
