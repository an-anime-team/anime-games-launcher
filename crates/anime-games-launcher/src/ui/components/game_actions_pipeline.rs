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

#[derive(Debug)]
pub enum GameActionsPipelineFactoryMsg {
    SetProgress {
        text: String,
        fraction: f64
    },

    SetProgressVisible(bool),
    SetFinished(bool)
}

#[derive(Debug, Clone, PartialEq)]
pub struct GameActionsPipelineFactory {
    pub title: String,
    pub progress_text: String,
    pub progress_fraction: f64,
    pub is_progress_visible: bool,
    pub is_finished: bool
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for GameActionsPipelineFactory {
    type Init = Self;
    type Input = GameActionsPipelineFactoryMsg;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = adw::PreferencesGroup;

    view! {
        #[root]
        adw::ActionRow {
            set_title: &self.title,

            add_suffix = &gtk::ProgressBar {
                set_valign: gtk::Align::Center,

                set_show_text: true,

                #[watch]
                set_visible: !self.is_finished && self.is_progress_visible,

                #[watch]
                set_text: Some(&self.progress_text),

                #[watch]
                set_fraction: self.progress_fraction
            },

            add_suffix = &gtk::Image {
                #[watch]
                set_visible: self.is_finished,

                set_icon_name: Some("emblem-ok-symbolic")
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

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncFactorySender<Self>
    ) {
        match msg {
            GameActionsPipelineFactoryMsg::SetProgress { text, fraction } => {
                self.progress_text = text;
                self.progress_fraction = fraction;
            }

            GameActionsPipelineFactoryMsg::SetProgressVisible(is_visible) => {
                self.is_progress_visible = is_visible;
            }

            GameActionsPipelineFactoryMsg::SetFinished(is_finished) => {
                self.is_finished = is_finished;
            }
        }
    }
}
