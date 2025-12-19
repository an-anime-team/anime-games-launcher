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

use std::sync::Arc;
use std::collections::VecDeque;

use adw::prelude::*;
use relm4::prelude::*;

use agl_games::engine::ActionsPipeline;

use crate::consts;
use crate::config;
use crate::ui::components::graph::{Graph, GraphInit, GraphMsg};
use crate::ui::components::game_actions_schedule::GameActionsScheduleFactory;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PipelineInfo {
    pub game_title: String,
    pub pipeline_title: String,
    pub pipeline_description: Option<String>
}

#[derive(Debug)]
pub struct DownloadsPage {
    graph: AsyncController<Graph>,
    actions_schedule: AsyncFactoryVecDeque<GameActionsScheduleFactory>,

    current_pipeline: Option<PipelineInfo>,
    scheduled_pipelines: VecDeque<PipelineInfo>
}

#[derive(Debug, Clone)]
pub enum DownloadsPageMsg {
    ScheduleGameActionsPipeline {
        game_index: usize,
        game_title: String,
        actions_pipeline: Arc<ActionsPipeline>
    }
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

                model.actions_schedule.widget() {
                    set_title: "Schedule"
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

            actions_schedule: AsyncFactoryVecDeque::builder()
                .launch_default()
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
            DownloadsPageMsg::ScheduleGameActionsPipeline {
                game_index,
                game_title,
                actions_pipeline
            } => {
                let lang = config::get().language();

                let pipeline_title = match &lang {
                    Ok(lang) => actions_pipeline.title().translate(lang),
                    Err(_)   => actions_pipeline.title().default_translation()
                };

                let pipeline_description = actions_pipeline.description()
                    .map(|description| {
                        match &lang {
                            Ok(lang) => description.translate(lang),
                            Err(_)   => description.default_translation()
                        }
                    })
                    .map(String::from);

                dbg!(&game_title);

                self.actions_schedule.guard().push_back(GameActionsScheduleFactory {
                    game_title: game_title.clone(),
                    pipeline_title: pipeline_title.to_string()
                });

                self.scheduled_pipelines.push_back(PipelineInfo {
                    game_title,
                    pipeline_title: pipeline_title.to_string(),
                    pipeline_description
                });
            }
        }
    }
}
