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

use super::card::{CardComponent, CardComponentOutput};

#[derive(Debug)]
pub enum CardsGridOutput {
    Clicked(DynamicIndex)
}

#[derive(Debug)]
pub struct CardsGrid {
    pub card: AsyncController<CardComponent>
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for CardsGrid {
    type Init = CardComponent;
    type Input = ();
    type Output = CardsGridOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        #[root]
        gtk::Box {
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            self.card.widget(),
        }
    }

    async fn init_model(
        init: Self::Init,
        index: &DynamicIndex,
        sender: AsyncFactorySender<Self>
    ) -> Self {
        let index = index.to_owned();

        Self {
            card: CardComponent::builder()
                .launch(init)
                .forward(sender.output_sender(), move |msg| {
                    match msg {
                        CardComponentOutput::Clicked => CardsGridOutput::Clicked(index.clone())
                    }
                })
        }
    }
}
