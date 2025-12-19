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
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::VecDeque;

use adw::prelude::*;
use relm4::prelude::*;

use agl_core::tasks;
use agl_games::engine::ActionsPipeline;

use crate::consts;
use crate::config;
use crate::ui::dialogs;
use crate::ui::components::graph::{Graph, GraphInit, GraphMsg};
use crate::ui::components::game_actions_pipeline::{
    GameActionsPipelineFactory, GameActionsPipelineFactoryMsg
};
use crate::ui::components::game_actions_schedule::GameActionsScheduleFactory;

#[derive(Debug, Clone)]
struct ScheduledPipelineInfo {
    pub game_title: String,
    pub pipeline_title: String,
    pub pipeline_description: Option<String>,
    pub pipeline: Arc<ActionsPipeline>,
    pub index: DynamicIndex
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CurrentPipelineInfo {
    pub game_title: String,
    pub pipeline_title: String,
    pub pipeline_description: Option<String>
}

#[derive(Debug)]
pub struct DownloadsPage {
    graph: AsyncController<Graph>,
    current_pipeline_factory: AsyncFactoryVecDeque<GameActionsPipelineFactory>,
    scheduled_pipelines_factory: AsyncFactoryVecDeque<GameActionsScheduleFactory>,

    current_pipeline: Option<CurrentPipelineInfo>,
    scheduled_pipelines: VecDeque<ScheduledPipelineInfo>
}

#[derive(Debug, Clone)]
pub enum DownloadsPageMsg {
    ScheduleGameActionsPipeline {
        game_index: usize,
        game_title: String,
        actions_pipeline: Arc<ActionsPipeline>
    },

    UpdateSchedule,

    SetCurrentPipelineActionProgress {
        action_number: usize,
        text: String,
        fraction: f64
    },

    AddGraphPoint(u64)
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
                    #[watch]
                    set_visible: model.current_pipeline.is_some(),

                    adw::Clamp {
                        set_hexpand: true,

                        model.graph.widget() {
                            set_halign: gtk::Align::Center
                        }
                    }
                },

                model.current_pipeline_factory.widget().clone() -> adw::PreferencesGroup {
                    #[watch]
                    set_visible: model.current_pipeline.is_some(),

                    #[watch]
                    set_title?: model.current_pipeline.as_ref()
                        .map(|info| info.pipeline_title.as_str()),

                    #[wrap(Some)]
                    set_header_suffix = &gtk::Label {
                        #[watch]
                        set_label?: model.current_pipeline.as_ref()
                            .map(|info| info.game_title.as_str())
                    }
                },

                model.scheduled_pipelines_factory.widget() {
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

            current_pipeline_factory: AsyncFactoryVecDeque::builder()
                .launch_default()
                .detach(),

            scheduled_pipelines_factory: AsyncFactoryVecDeque::builder()
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
        sender: AsyncComponentSender<Self>
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
                    Err(_) => actions_pipeline.title().default_translation()
                };

                let pipeline_description = actions_pipeline.description()
                    .map(|description| {
                        match &lang {
                            Ok(lang) => description.translate(lang),
                            Err(_) => description.default_translation()
                        }
                    })
                    .map(String::from);

                let index = self.scheduled_pipelines_factory.guard()
                    .push_back(GameActionsScheduleFactory {
                        game_title: game_title.clone(),
                        pipeline_title: pipeline_title.to_string(),
                        pipeline_description: pipeline_description.clone()
                    });

                self.scheduled_pipelines.push_back(ScheduledPipelineInfo {
                    game_title,
                    pipeline_title: pipeline_title.to_string(),
                    pipeline_description,
                    pipeline: actions_pipeline,
                    index
                });

                sender.input(DownloadsPageMsg::UpdateSchedule);
            }

            DownloadsPageMsg::UpdateSchedule => {
                if self.current_pipeline.is_none()
                    && let Some(pipeline_info) = self.scheduled_pipelines.pop_front()
                {
                    self.scheduled_pipelines_factory.guard()
                        .remove(pipeline_info.index.current_index());

                    self.current_pipeline = Some(CurrentPipelineInfo {
                        game_title: pipeline_info.game_title,
                        pipeline_title: pipeline_info.pipeline_title,
                        pipeline_description: pipeline_info.pipeline_description
                    });

                    let lang = config::get().language().ok();

                    let mut guard = self.current_pipeline_factory.guard();
                    let mut actions = Vec::new();

                    guard.clear();

                    for action in pipeline_info.pipeline.actions() {
                        let title = match &lang {
                            Some(lang) => action.title().translate(lang),
                            None => action.title().default_translation()
                        };

                        let index = guard.push_back(GameActionsPipelineFactory {
                            title: title.to_string(),
                            progress_fraction: 0.0,
                            progress_text: String::new()
                        });

                        actions.push((action.clone(), index));
                    }

                    tasks::spawn_blocking(move || {
                        for (action, index) in actions {
                            let result = {
                                let lang = lang.clone();
                                let action_number = index.current_index();
                                let sender = sender.clone();

                                let last_current = AtomicU64::new(0);

                                action.before(move |progress| {
                                    let fraction = progress.fraction();

                                    let text = progress.format().ok()
                                        .flatten()
                                        .map(|text| {
                                            let text = match &lang {
                                                Some(lang) => text.translate(lang),
                                                None => text.default_translation()
                                            };

                                            text.to_string()
                                        })
                                        .unwrap_or_else(|| {
                                            format!("{:.2}%", fraction * 100.0)
                                        });

                                    // TODO: percent change per second
                                    let diff = progress.current()
                                        .checked_sub(last_current.load(Ordering::Relaxed))
                                        .unwrap_or_default();

                                    last_current.store(progress.current(), Ordering::Relaxed);

                                    sender.input(DownloadsPageMsg::AddGraphPoint(diff));

                                    sender.input(DownloadsPageMsg::SetCurrentPipelineActionProgress {
                                        action_number,
                                        text,
                                        fraction
                                    });
                                })
                            };

                            match result {
                                Ok(Some(true)) | Ok(None) => {
                                    let lang = lang.clone();
                                    let action_number = index.current_index();
                                    let sender = sender.clone();

                                    let last_current = AtomicU64::new(0);

                                    let result = action.perform(move |progress| {
                                        let fraction = progress.fraction();

                                        let text = progress.format().ok()
                                            .flatten()
                                            .map(|text| {
                                                let text = match &lang {
                                                    Some(lang) => text.translate(lang),
                                                    None => text.default_translation()
                                                };

                                                text.to_string()
                                            })
                                            .unwrap_or_else(|| {
                                                format!("{:.2}%", fraction * 100.0)
                                            });

                                        // TODO: percent change per second
                                        let diff = progress.current()
                                            .checked_sub(last_current.load(Ordering::Relaxed))
                                            .unwrap_or_default();

                                        last_current.store(progress.current(), Ordering::Relaxed);

                                        sender.input(DownloadsPageMsg::AddGraphPoint(diff));

                                        sender.input(DownloadsPageMsg::SetCurrentPipelineActionProgress {
                                            action_number,
                                            text,
                                            fraction
                                        });
                                    });

                                    match result {
                                        Ok(true) => (),

                                        Ok(false) => {
                                            tracing::error!("pipeline action returned error response");

                                            dialogs::error(
                                                "Actions pipeline failed",
                                                "One of the pipeline actions returned false"
                                            );

                                            break;
                                        }

                                        Err(err) => {
                                            tracing::error!(?err, "failed to perform pipeline action");

                                            dialogs::error("Failed to perform pipeline action", err.to_string());

                                            break;
                                        }
                                    }
                                }

                                Ok(Some(false)) => (),

                                Err(err) => {
                                    tracing::error!(?err, "failed to perform pipeline action");

                                    dialogs::error("Failed to perform pipeline action", err.to_string());

                                    break;
                                }
                            }

                            sender.input(DownloadsPageMsg::SetCurrentPipelineActionProgress {
                                action_number: index.current_index(),
                                text: String::new(),
                                fraction: 1.0
                            });
                        }
                    });
                }
            }

            DownloadsPageMsg::SetCurrentPipelineActionProgress {
                action_number,
                text,
                fraction
            } => {
                self.current_pipeline_factory.send(
                    action_number,
                    GameActionsPipelineFactoryMsg::SetProgress { text, fraction }
                );
            }

            DownloadsPageMsg::AddGraphPoint(point) => {
                self.graph.emit(GraphMsg::AddPoint(point));
            }
        }
    }
}
