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

use std::collections::VecDeque;

use adw::prelude::*;
use relm4::prelude::*;

use crate::consts;
use crate::ui::components::graph::{Graph, GraphInit, GraphMsg};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PipelineInfo {
    pub game_title: String,
    pub pipeline_title: String,
    pub pipeline_description: Option<String>
}

#[derive(Debug)]
pub struct DownloadsPage {
    graph: AsyncController<Graph>,

    current_pipeline: Option<PipelineInfo>,
    scheduled_pipelines: VecDeque<PipelineInfo>
}

#[derive(Debug, Clone)]
pub enum DownloadsPageMsg {

}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for DownloadsPage {
    type Init = ();
    type Input = DownloadsPageMsg;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_vexpand: true,
            set_hexpand: true,

            set_orientation: gtk::Orientation::Vertical,

            adw::StatusPage {
                set_vexpand: true,
                set_hexpand: true,

                set_icon_name: Some(consts::APP_ID),

                set_title: "No actions scheduled",

                #[watch]
                set_visible: model.current_pipeline.is_none() && model.scheduled_pipelines.is_empty()
            },

            adw::PreferencesPage {
                #[watch]
                set_visible: model.current_pipeline.is_some() || !model.scheduled_pipelines.is_empty(),

                adw::PreferencesGroup {
                    adw::Clamp {
                        set_hexpand: true,

                        model.graph.widget() {
                            set_halign: gtk::Align::Center
                        }
                    }
                },

                adw::PreferencesGroup {
                    set_title: "Update game",

                    #[wrap(Some)]
                    set_header_suffix = &gtk::Label {
                        set_label: "Genshin Impact"
                    },

                    adw::ActionRow {
                        set_title: "Download",

                        add_suffix = &gtk::Image {
                            set_icon_name: Some("emblem-ok-symbolic")
                        }
                    },

                    adw::ActionRow {
                        set_title: "Extract",

                        add_suffix = &gtk::ProgressBar {
                            set_valign: gtk::Align::Center,

                            set_show_text: true,

                            set_text: Some("13 MB/s"),
                            set_fraction: 0.65
                        }
                    },

                    adw::ActionRow {
                        set_title: "Verify"
                    }
                },

                adw::PreferencesGroup {
                    set_title: "Schedule",

                    adw::ActionRow {
                        set_title: "Update game",

                        add_suffix = &gtk::Label {
                            set_label: "Honkai: Star Rail"
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
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            graph: Graph::builder()
                .launch(GraphInit {
                    width: 600,
                    height: 180,
                    window_size: 60,
                    color: (1.0, 0.0, 0.0)
                })
                .detach(),

            current_pipeline: None,
            scheduled_pipelines: VecDeque::new()
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncComponentSender<Self>
    ) {
        match msg {

        }
    }
}
