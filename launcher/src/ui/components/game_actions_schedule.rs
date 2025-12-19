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
pub struct GameActionsScheduleFactory {
    pub game_title: String,
    pub pipeline_title: String
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for GameActionsScheduleFactory {
    type Init = Self;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = adw::PreferencesGroup;

    view! {
        #[root]
        adw::ActionRow {
            set_title: &self.pipeline_title,

            add_suffix = &gtk::Label {
                set_label: &self.game_title
            },

            add_suffix = &gtk::Button {
                set_valign: gtk::Align::Center,

                add_css_class: "flat",

                adw::ButtonContent {
                    set_icon_name: "window-close-symbolic"
                }
            }
        }
    }

    #[inline]
    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        init
    }

    // async fn update(
    //     &mut self,
    //     msg: Self::Input,
    //     _sender: AsyncFactorySender<Self>
    // ) {
    //     match msg {

    //     }
    // }
}
