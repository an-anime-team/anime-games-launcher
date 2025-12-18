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

use crate::utils;
// use crate::ui::components::downloads_row::{
//     DownloadsRow, DownloadsRowFactory, DownloadsRowFactoryOutput, DownloadsRowInit,
// };
use crate::ui::components::graph::{Graph, GraphInit, GraphMsg};

#[derive(Debug)]
pub struct DownloadsPage {
    graph: AsyncController<Graph>,
    // active: AsyncController<DownloadsRow>,
    // scheduled: AsyncFactoryVecDeque<DownloadsRowFactory>,
    // state: DownloadsAppState,

    // Graph
    // graph_speed: u64,
    // graph_avg_speed: u64,
    // graph_total: u64,
    // graph_elapsed: u64
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
        adw::PreferencesPage {
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
            },

            // adw::PreferencesGroup {
            //     #[watch]
            //     set_visible: match model.state {
            //         DownloadsAppState::None => false,
            //         _ => true,
            //     },

            //     #[watch]
            //     set_title: match model.state {
            //         DownloadsAppState::None => "",
            //         DownloadsAppState::Downloading => "Downloading",
            //         DownloadsAppState::Extracting => "Extracting",
            //         DownloadsAppState::StreamUnpacking => "Stream unpacking",
            //         DownloadsAppState::Verifying => "Verifying",
            //     },

            //     gtk::Box {
            //         set_orientation: gtk::Orientation::Horizontal,
            //         set_spacing: 16,

            //         adw::PreferencesGroup {
            //             adw::ActionRow {
            //                 set_title: "Current speed",

            //                 #[watch]
            //                 set_subtitle: &format!("{}/s", pretty_bytes(model.speed).0),
            //             }
            //         },

            //         adw::PreferencesGroup {
            //             adw::ActionRow {
            //                 set_title: "Average speed",

            //                 #[watch]
            //                 set_subtitle: &format!("{}/s", pretty_bytes(model.avg_speed).0),
            //             }
            //         },

            //         adw::PreferencesGroup {
            //             adw::ActionRow {
            //                 set_title: "Time elapsed",

            //                 #[watch]
            //                 set_subtitle: &pretty_seconds(model.elapsed),
            //             }
            //         },

            //         adw::PreferencesGroup {
            //             adw::ActionRow {
            //                 set_title: "Current ETA",
            //                 set_subtitle: "amogus",
            //             }
            //         },

            //         adw::PreferencesGroup {
            //             adw::ActionRow {
            //                 #[watch]
            //                 set_title: match model.state {
            //                     DownloadsAppState::None => "",
            //                     DownloadsAppState::Downloading => "Total download",
            //                     DownloadsAppState::Extracting => "Total extracted",
            //                     DownloadsAppState::StreamUnpacking => "Total unpacked",
            //                     DownloadsAppState::Verifying => "Total verified",
            //                 },

            //                 #[watch]
            //                 set_subtitle: &pretty_bytes(model.total).0.to_string(),
            //             }
            //         },
            //     }
            // },

            // adw::PreferencesGroup {
            //     set_title: "Active",

            //     model.active.widget(),
            // },

            // model.scheduled.widget() {
            //     set_title: "Scheduled",
            // },
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
                .detach()
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
