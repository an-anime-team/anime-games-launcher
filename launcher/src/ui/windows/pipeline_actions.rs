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
use std::cell::Cell;
use std::time::{Instant, Duration};

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

const GRAPH_DIFF_INTERVAL: Duration = Duration::from_millis(500);
const GRAPH_DIFF_PRECISION: f64 = 1_000_000.0;

#[derive(Debug, Clone)]
pub enum PipelineActionsWindowInput {
    SetActionsPipeline {
        game_index: usize,
        game_title: String,
        actions_pipeline: Arc<ActionsPipeline>
    },

    SetProgress {
        action_number: usize,
        text: String,
        fraction: f64
    },

    SetFinished {
        action_number: usize,
        is_finished: bool
    },

    AddGraphPoint(u64),

    EmitClose
}

#[derive(Debug, Clone)]
pub enum PipelineActionsWindowOutput {
    UpdateGameInfo(usize)
}

#[derive(Debug)]
pub struct PipelineActionsWindow {
    graph: AsyncController<Graph>,
    pipeline_actions: AsyncFactoryVecDeque<GameActionsPipelineFactory>,

    window: Option<adw::Dialog>,

    game_index: Option<usize>,
    game_title: Option<String>,

    pipeline_title: Option<String>,
    pipeline_description: Option<String>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for PipelineActionsWindow {
    type Init = ();
    type Input = PipelineActionsWindowInput;
    type Output = PipelineActionsWindowOutput;

    view! {
        #[root]
        _window = adw::Dialog {
            set_size_request: (800, 600),
            set_can_close: false,

            add_css_class?: consts::APP_DEBUG.then_some("devel"),

            #[watch]
            set_title?: &model.game_title,

            #[wrap(Some)]
            set_child = &gtk::Box {
                set_vexpand: true,
                set_hexpand: true,

                set_orientation: gtk::Orientation::Vertical,

                gtk::Label {
                    set_margin_top: 16,
                    set_margin_bottom: 16,

                    #[watch]
                    set_label?: &model.game_title
                },

                adw::PreferencesPage {
                    adw::PreferencesGroup {
                        adw::Clamp {
                            set_hexpand: true,

                            model.graph.widget() {
                                set_halign: gtk::Align::Center
                            }
                        }
                    },

                    model.pipeline_actions.widget().clone() -> adw::PreferencesGroup {
                        #[watch]
                        set_title?: &model.pipeline_title,

                        #[watch]
                        set_description: model.pipeline_description.as_deref()
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
        let accent_color = adw::StyleManager::default()
            .accent_color_rgba();

        let mut model = Self {
            graph: Graph::builder()
                .launch(GraphInit {
                    width: 600,
                    height: 180,
                    window_size: 60,
                    color: (
                        accent_color.red() as f64,
                        accent_color.green() as f64,
                        accent_color.blue() as f64
                    )
                })
                .detach(),

            pipeline_actions: AsyncFactoryVecDeque::builder()
                .launch_default()
                .detach(),

            window: None,

            game_index: None,
            game_title: None,

            pipeline_title: None,
            pipeline_description: None
        };

        let widgets = view_output!();

        model.window = Some(widgets._window.clone());

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>
    ) {
        match msg {
            PipelineActionsWindowInput::SetActionsPipeline {
                game_index,
                game_title,
                actions_pipeline
            } => {
                let lang = config::get().language().ok();

                let pipeline_title = match &lang {
                    Some(lang) => actions_pipeline.title().translate(lang),
                    None => actions_pipeline.title().default_translation()
                };

                let pipeline_description = actions_pipeline.description()
                    .map(|description| {
                        match &lang {
                            Some(lang) => description.translate(lang),
                            None => description.default_translation()
                        }
                    })
                    .map(String::from);

                self.game_index = Some(game_index);
                self.game_title = Some(game_title);

                self.pipeline_title = Some(pipeline_title.to_string());
                self.pipeline_description = pipeline_description;

                let mut guard = self.pipeline_actions.guard();
                let mut actions = Vec::new();

                guard.clear();

                for action in actions_pipeline.actions() {
                    let title = match &lang {
                        Some(lang) => action.title().translate(lang),
                        None => action.title().default_translation()
                    };

                    let index = guard.push_back(GameActionsPipelineFactory {
                        title: title.to_string(),
                        progress_fraction: 0.0,
                        progress_text: String::new(),
                        is_finished: false
                    });

                    actions.push((action.clone(), index));
                }

                drop(guard);

                tasks::spawn_blocking(move || {
                    for (action, index) in actions {
                        let result = {
                            let lang = lang.clone();
                            let action_number = index.current_index();
                            let sender = sender.clone();

                            let last_update = Cell::new((
                                Instant::now(),
                                0.0
                            ));

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

                                let (instant, last_fraction) = last_update.get();

                                if instant.elapsed() > GRAPH_DIFF_INTERVAL {
                                    let curr_fraction = progress.fraction();

                                    sender.input(PipelineActionsWindowInput::AddGraphPoint(
                                        ((curr_fraction - last_fraction) * GRAPH_DIFF_PRECISION) as u64
                                    ));

                                    last_update.set((
                                        Instant::now(),
                                        curr_fraction
                                    ));
                                }

                                sender.input(PipelineActionsWindowInput::SetProgress {
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

                                let last_update = Cell::new((
                                    Instant::now(),
                                    0.0
                                ));

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

                                    let (instant, last_fraction) = last_update.get();

                                    if instant.elapsed() > GRAPH_DIFF_INTERVAL {
                                        let curr_fraction = progress.fraction();

                                        sender.input(PipelineActionsWindowInput::AddGraphPoint(
                                            ((curr_fraction - last_fraction) * GRAPH_DIFF_PRECISION) as u64
                                        ));

                                        last_update.set((
                                            Instant::now(),
                                            curr_fraction
                                        ));
                                    }

                                    sender.input(PipelineActionsWindowInput::SetProgress {
                                        action_number,
                                        text,
                                        fraction
                                    });
                                });

                                if let Err(err) = result {
                                    tracing::error!(?err, "failed to perform pipeline action");

                                    dialogs::error("Failed to perform pipeline action", err.to_string());

                                    break;
                                }
                            }

                            Ok(Some(false)) => (),

                            Err(err) => {
                                tracing::error!(?err, "failed to perform pipeline action");

                                dialogs::error("Failed to perform pipeline action", err.to_string());

                                break;
                            }
                        }

                        sender.input(PipelineActionsWindowInput::SetProgress {
                            action_number: index.current_index(),
                            text: String::new(),
                            fraction: 1.0
                        });

                        sender.input(PipelineActionsWindowInput::SetFinished {
                            action_number: index.current_index(),
                            is_finished: true
                        });
                    }

                    sender.input(PipelineActionsWindowInput::EmitClose);
                });
            }

            PipelineActionsWindowInput::SetProgress {
                action_number,
                text,
                fraction
            } => {
                self.pipeline_actions.send(
                    action_number,
                    GameActionsPipelineFactoryMsg::SetProgress { text, fraction }
                );
            }

            PipelineActionsWindowInput::SetFinished {
                action_number,
                is_finished
            } => {
                self.pipeline_actions.send(
                    action_number,
                    GameActionsPipelineFactoryMsg::SetFinished(is_finished)
                );
            }

            PipelineActionsWindowInput::AddGraphPoint(point) => {
                self.graph.emit(GraphMsg::AddPoint(point));
            }

            PipelineActionsWindowInput::EmitClose => {
                if let Some(index) = self.game_index {
                    let _ = sender.output(PipelineActionsWindowOutput::UpdateGameInfo(index));
                }

                if let Some(window) = &self.window {
                    window.force_close();
                }
            }
        }
    }
}
