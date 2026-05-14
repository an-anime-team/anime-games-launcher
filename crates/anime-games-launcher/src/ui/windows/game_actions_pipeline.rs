// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@vk.com>
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

use adw::prelude::*;
use relm4::prelude::*;

use agl_core::tasks;
use agl_games::api::ActionsPipeline;

use crate::{consts, config, i18n};
use crate::ui::dialogs;
use crate::ui::components::graph_progress_group::{
    GraphProgressGroup, GraphProgressGroupInit, GraphProgressGroupMsg
};

#[derive(Debug, Clone)]
pub enum GameActionsPipelineWindowInput {
    SetActionsPipeline {
        game_name: String,
        game_title: String,
        actions_pipeline: Arc<ActionsPipeline>
    },

    MarkStarted {
        name: String
    },

    SetProgress {
        name: String,
        text: Option<String>,
        fraction: f64
    },

    MarkFinished {
        name: String
    },

    EmitClose
}

#[derive(Debug, Clone)]
pub enum GameActionsPipelineWindowOutput {
    UpdateGameInfo(String)
}

#[derive(Debug)]
pub struct GameActionsPipelineWindow {
    graph_group: AsyncController<GraphProgressGroup>,

    window: adw::Dialog,

    game_name: Option<String>,
    game_title: Option<String>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameActionsPipelineWindow {
    type Init = ();
    type Input = GameActionsPipelineWindowInput;
    type Output = GameActionsPipelineWindowOutput;

    view! {
        #[root]
        adw::Dialog {
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
                    set_label: match &model.game_title {
                        Some(title) => title,
                        None => ""
                    }
                },

                model.graph_group.widget(),
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            graph_group: GraphProgressGroup::builder()
                .launch(GraphProgressGroupInit {
                    title: None,
                    description: None
                })
                .detach(),

            window: root.clone(),

            game_name: None,
            game_title: None
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
            GameActionsPipelineWindowInput::SetActionsPipeline {
                game_name,
                game_title,
                actions_pipeline
            } => {
                let lang = config::get().language().ok();

                let title = match &lang {
                    Some(lang) => actions_pipeline.title().translate(lang),
                    None => actions_pipeline.title().default_translation()
                };

                let description = actions_pipeline.description()
                    .map(|description| {
                        match &lang {
                            Some(lang) => description.translate(lang),
                            None => description.default_translation()
                        }
                    })
                    .map(String::from);

                self.graph_group.emit(GraphProgressGroupMsg::ClearGraph);
                self.graph_group.emit(GraphProgressGroupMsg::ClearProgressRows);

                self.graph_group.emit(GraphProgressGroupMsg::SetTitle(Some(title.to_string())));
                self.graph_group.emit(GraphProgressGroupMsg::SetDescription(description));

                self.game_name = Some(game_name);
                self.game_title = Some(game_title);

                let mut actions = Vec::new();

                for (i, action) in actions_pipeline.actions().iter().enumerate() {
                    let name = i.to_string();

                    let title = match &lang {
                        Some(lang) => action.title().translate(lang),
                        None => action.title().default_translation()
                    };

                    let description = action.description()
                        .map(|description| {
                            match &lang {
                                Some(lang) => description.translate(lang),
                                None => description.default_translation()
                            }
                        })
                        .map(String::from);

                    self.graph_group.emit(GraphProgressGroupMsg::AddProgressRow {
                        name: name.clone(),
                        title: title.to_string(),
                        description
                    });

                    actions.push((name, action.clone()));
                }

                tasks::spawn_blocking(move || {
                    for (name, action) in actions {
                        sender.input(GameActionsPipelineWindowInput::MarkStarted {
                            name: name.clone()
                        });

                        let result = {
                            let lang = lang.clone();
                            let name = name.clone();
                            let sender = sender.clone();

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

                                sender.input(GameActionsPipelineWindowInput::SetProgress {
                                    name: name.clone(),
                                    text: Some(text),
                                    fraction
                                });
                            })
                        };

                        match result {
                            Ok(Some(true)) | Ok(None) => {
                                let lang = lang.clone();
                                let name = name.clone();
                                let sender = sender.clone();

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

                                    sender.input(GameActionsPipelineWindowInput::SetProgress {
                                        name: name.clone(),
                                        text: Some(text),
                                        fraction
                                    });
                                });

                                if let Err(err) = result {
                                    tracing::error!(?err, "failed to perform pipeline action");

                                    dialogs::error(
                                        i18n!("failed_perform_pipeline_action")
                                            .unwrap_or("Failed to perform pipeline action"),
                                        err.to_string()
                                    );

                                    break;
                                }
                            }

                            Ok(Some(false)) => (),

                            Err(err) => {
                                tracing::error!(?err, "failed to perform pipeline action");

                                dialogs::error(
                                    i18n!("failed_perform_pipeline_action")
                                        .unwrap_or("Failed to perform pipeline action"),
                                    err.to_string()
                                );

                                break;
                            }
                        }

                        sender.input(GameActionsPipelineWindowInput::SetProgress {
                            name: name.clone(),
                            text: None,
                            fraction: 1.0
                        });

                        sender.input(GameActionsPipelineWindowInput::MarkFinished {
                            name
                        });
                    }

                    sender.input(GameActionsPipelineWindowInput::EmitClose);
                });
            }

            GameActionsPipelineWindowInput::MarkStarted { name } => {
                self.graph_group.emit(GraphProgressGroupMsg::MarkStarted {
                    name
                });
            }

            GameActionsPipelineWindowInput::SetProgress {
                name,
                text,
                fraction
            } => {
                self.graph_group.emit(GraphProgressGroupMsg::SetProgress {
                    name,
                    text,
                    fraction
                });
            }

            GameActionsPipelineWindowInput::MarkFinished { name } => {
                self.graph_group.emit(GraphProgressGroupMsg::MarkFinished {
                    name
                });
            }

            GameActionsPipelineWindowInput::EmitClose => {
                self.graph_group.emit(GraphProgressGroupMsg::ClearGraph);
                self.graph_group.emit(GraphProgressGroupMsg::ClearProgressRows);

                if let Some(name) = &self.game_name {
                    let _ = sender.output(GameActionsPipelineWindowOutput::UpdateGameInfo(
                        name.clone()
                    ));
                }

                self.window.force_close();
            }
        }
    }
}
