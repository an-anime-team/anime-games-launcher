// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2026  Nikita Podvirnyi <krypt0nn@vk.com>
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameToolButtonInput {
    EmitClick
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameToolButtonOutput {
    Clicked(DynamicIndex)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameToolButtonInit {
    pub title: String,
    pub description: Option<String>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameToolButtonFactory {
    title: String,
    description: Option<String>,
    index: DynamicIndex
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for GameToolButtonFactory {
    type Init = GameToolButtonInit;
    type Input = GameToolButtonInput;
    type Output = GameToolButtonOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        gtk::Button {
            add_css_class: "pill",

            #[watch]
            set_label: &self.title,

            #[watch]
            set_tooltip: {
                match &self.description {
                    Some(description) => description.as_str(),
                    None => ""
                }
            },

            connect_clicked => GameToolButtonInput::EmitClick
        }
    }

    #[inline]
    async fn init_model(
        init: Self::Init,
        index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>
    ) -> Self {
        Self {
            title: init.title,
            description: init.description,
            index: index.clone()
        }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncFactorySender<Self>
    ) {
        match msg {
            GameToolButtonInput::EmitClick => {
                let _ = sender.output(GameToolButtonOutput::Clicked(
                    self.index.clone()
                ));
            }
        }
    }
}
