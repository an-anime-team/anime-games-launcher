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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameActionsScheduleFactoryInit {
    pub game_title: String,
    pub pipeline_title: String,
    pub pipeline_description: Option<String>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameActionsScheduleFactoryInput {
    EmitRemove
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameActionsScheduleFactoryOutput {
    Remove(DynamicIndex)
}

#[derive(Debug, Clone)]
pub struct GameActionsScheduleFactory {
    index: DynamicIndex,
    game_title: String,
    pipeline_title: String,
    pipeline_description: Option<String>
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for GameActionsScheduleFactory {
    type Init = GameActionsScheduleFactoryInit;
    type Input = GameActionsScheduleFactoryInput;
    type Output = GameActionsScheduleFactoryOutput;
    type CommandOutput = ();
    type ParentWidget = adw::PreferencesGroup;

    view! {
        #[root]
        adw::ActionRow {
            set_title: &self.pipeline_title,
            set_subtitle?: &self.pipeline_description,

            add_suffix = &gtk::Label {
                set_label: &self.game_title
            },

            add_suffix = &gtk::Button {
                set_valign: gtk::Align::Center,

                add_css_class: "flat",

                adw::ButtonContent {
                    set_icon_name: "window-close-symbolic"
                },

                connect_clicked => GameActionsScheduleFactoryInput::EmitRemove
            }
        }
    }

    #[inline]
    async fn init_model(
        init: Self::Init,
        index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            index: index.clone(),
            game_title: init.game_title,
            pipeline_title: init.pipeline_title,
            pipeline_description: init.pipeline_description
        }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncFactorySender<Self>
    ) {
        match msg {
            GameActionsScheduleFactoryInput::EmitRemove => {
                let _ = sender.output(GameActionsScheduleFactoryOutput::Remove(self.index.clone()));
            }
        }
    }
}
