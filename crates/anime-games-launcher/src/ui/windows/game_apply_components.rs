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
use agl_games::api::{GameVariant, GameIntegration, ProgressReport};

use crate::{consts, config, i18n};
use crate::ui::dialogs;
use crate::ui::components::graph_progress_group::{
    GraphProgressGroup, GraphProgressGroupInit, GraphProgressGroupMsg
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplyComponentInfo {
    pub name: String,
    pub title: String
}

#[derive(Debug, Clone)]
pub enum GameApplyComponentsWindowInput {
    SetComponents {
        game_variant: GameVariant,
        game_integration: Arc<GameIntegration>,

        game_name: String,
        game_title: String,

        install_components: Box<[ApplyComponentInfo]>,
        uninstall_components: Box<[ApplyComponentInfo]>,

        /// Delete game package after applying these components.
        delete_game_package: bool
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
pub enum GameApplyComponentsWindowOutput {
    UpdateGameInfo(String),
    DeleteGamePackage(String)
}

#[derive(Debug)]
pub struct GameApplyComponentsWindow {
    graph_group: AsyncController<GraphProgressGroup>,

    window: adw::Dialog,

    game_name: Option<String>,
    game_title: Option<String>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameApplyComponentsWindow {
    type Init = ();
    type Input = GameApplyComponentsWindowInput;
    type Output = GameApplyComponentsWindowOutput;

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
                    title: i18n!("game_components_apply_changes_title")
                        .map(String::from),
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
            GameApplyComponentsWindowInput::SetComponents {
                game_variant,
                game_integration,
                game_name,
                game_title,
                install_components,
                uninstall_components,
                delete_game_package
            } => {
                let lang = config::get().await
                    .language().ok();

                self.graph_group.emit(GraphProgressGroupMsg::ClearGraph);
                self.graph_group.emit(GraphProgressGroupMsg::ClearProgressRows);

                self.game_name = Some(game_name.clone());
                self.game_title = Some(game_title);

                if delete_game_package {
                    self.graph_group.emit(GraphProgressGroupMsg::SetTitle(Some(
                        i18n!("game_components_uninstall_all_title")
                            .map(String::from)
                            .unwrap_or_else(|| String::from("Uninstall game components"))
                    )));
                } else {
                    self.graph_group.emit(GraphProgressGroupMsg::SetTitle(Some(
                        i18n!("game_components_apply_changes_title")
                            .map(String::from)
                            .unwrap_or_else(|| String::from("Apply components changes"))
                    )));
                }

                let mut actions = Vec::new();

                for component in uninstall_components {
                    let title = i18n!("game_component_uninstall_title", {
                        component => component.title
                    }).unwrap_or_else(|| {
                        format!("Uninstall {}", component.title)
                    });

                    self.graph_group.emit(GraphProgressGroupMsg::AddProgressRow {
                        name: component.name.clone(),
                        title,
                        description: None
                    });

                    actions.push((component.name, false));
                }

                for component in install_components {
                    let title = i18n!("game_component_install_title", {
                        component => component.title
                    }).unwrap_or_else(|| {
                        format!("Install {}", component.title)
                    });

                    self.graph_group.emit(GraphProgressGroupMsg::AddProgressRow {
                        name: component.name.clone(),
                        title,
                        description: None
                    });

                    actions.push((component.name, true));
                }

                tasks::spawn_blocking(move || {
                    for (name, is_install) in actions {
                        sender.input(GameApplyComponentsWindowInput::MarkStarted {
                            name: name.clone()
                        });

                        let updater = {
                            let sender = sender.clone();
                            let lang = lang.clone();
                            let name = name.clone();

                            move |progress: ProgressReport| {
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

                                sender.input(GameApplyComponentsWindowInput::SetProgress {
                                    name: name.clone(),
                                    text: Some(text),
                                    fraction
                                });
                            }
                        };

                        let mut result = if is_install {
                            game_integration.install_component(
                                &game_variant,
                                &name,
                                updater
                            )
                        } else {
                            game_integration.uninstall_component(
                                &game_variant,
                                &name,
                                updater
                            )
                        };

                        // If we've successfully installed/uninstalled the
                        // component then save its enabled/disabled state.
                        if result.is_ok() {
                            result = game_integration.set_component_enabled(
                                &game_variant,
                                &name,
                                is_install
                            );
                        }

                        if let Err(err) = result {
                            tracing::error!(
                                ?err,
                                component = ?name,
                                "failed to apply game component"
                            );

                            dialogs::error(
                                i18n!("failed_apply_game_component")
                                    .unwrap_or("Failed to apply game component"),
                                err.to_string()
                            );

                            break;
                        }

                        sender.input(GameApplyComponentsWindowInput::SetProgress {
                            name: name.clone(),
                            text: None,
                            fraction: 1.0
                        });

                        sender.input(GameApplyComponentsWindowInput::MarkFinished {
                            name
                        });
                    }

                    if delete_game_package {
                        let _ = sender.output(GameApplyComponentsWindowOutput::DeleteGamePackage(
                            game_name
                        ));
                    }

                    sender.input(GameApplyComponentsWindowInput::EmitClose);
                });
            }

            GameApplyComponentsWindowInput::MarkStarted { name } => {
                self.graph_group.emit(GraphProgressGroupMsg::MarkStarted {
                    name
                });
            }

            GameApplyComponentsWindowInput::SetProgress {
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

            GameApplyComponentsWindowInput::MarkFinished { name } => {
                self.graph_group.emit(GraphProgressGroupMsg::MarkFinished {
                    name
                });
            }

            GameApplyComponentsWindowInput::EmitClose => {
                self.graph_group.emit(GraphProgressGroupMsg::ClearGraph);
                self.graph_group.emit(GraphProgressGroupMsg::ClearProgressRows);

                if let Some(game_name) = &self.game_name {
                    let _ = sender.output(GameApplyComponentsWindowOutput::UpdateGameInfo(
                        game_name.clone()
                    ));
                }

                self.window.force_close();
            }
        }
    }
}
