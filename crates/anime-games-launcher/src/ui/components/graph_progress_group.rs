// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
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

use std::collections::HashMap;
use std::time::{Instant, Duration};

use adw::prelude::*;
use relm4::prelude::*;

use super::graph::{Graph, GraphInit, GraphMsg};
use super::progress_bar_group::{
    ProgressBarGroup, ProgressBarGroupMsg, ProgressBarGroupValue
};

const GRAPH_DIFF_INTERVAL: Duration = Duration::from_secs(1);
const GRAPH_DIFF_PRECISION: f64 = 1_000_000.0;
const GRAPH_POINTS_NUM: usize = 60;
const GRAPH_WMA_SIZE: usize = 5;
const GRAPH_WMA_DIVISOR: f64 = (GRAPH_WMA_SIZE as f64 + 1.0) / 2.0 * (GRAPH_WMA_SIZE as f64);

#[derive(Debug, Clone, PartialEq)]
struct ProgressRow {
    pub index: DynamicIndex,
    pub last_update: Instant,
    pub last_fraction: f64,
    pub graph_diffs_wma: [f64; GRAPH_WMA_SIZE]
}

#[derive(Debug, Clone, PartialEq)]
pub enum GraphProgressGroupMsg {
    SetTitle(Option<String>),
    SetDescription(Option<String>),

    AddProgressRow {
        name: String,
        title: String,
        description: Option<String>
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

    ClearGraph,
    ClearProgressRows
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GraphProgressGroupInit {
    /// Title of the progress group.
    pub title: Option<String>,

    /// Description of the progress group.
    pub description: Option<String>
}

/// A component that combines `Graph` with `ProgressBarGroup` to display the
/// overall progress on a graph.
#[derive(Debug)]
pub struct GraphProgressGroup {
    graph: AsyncController<Graph>,
    progress_group: AsyncFactoryVecDeque<ProgressBarGroup>,

    progress_rows: HashMap<String, ProgressRow>,

    title: Option<String>,
    description: Option<String>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GraphProgressGroup {
    type Init = GraphProgressGroupInit;
    type Input = GraphProgressGroupMsg;
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

            model.progress_group.widget().clone() -> adw::PreferencesGroup {
                #[watch]
                set_title: match &model.title {
                    Some(title) => title,
                    None => ""
                },

                #[watch]
                set_description: model.description.as_deref()
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>
    ) -> AsyncComponentParts<Self> {
        let accent_color = adw::StyleManager::default()
            .accent_color_rgba();

        let model = Self {
            graph: Graph::builder()
                .launch(GraphInit {
                    width: 600,
                    height: 180,
                    points_num: GRAPH_POINTS_NUM,
                    color: (
                        accent_color.red() as f64,
                        accent_color.green() as f64,
                        accent_color.blue() as f64
                    )
                })
                .detach(),

            progress_group: AsyncFactoryVecDeque::builder()
                .launch_default()
                .detach(),

            // Random capacity which I took from my head.
            progress_rows: HashMap::with_capacity(3),

            title: init.title,
            description: init.description
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
            GraphProgressGroupMsg::SetTitle(title) => {
                self.title = title;
            }

            GraphProgressGroupMsg::SetDescription(description) => {
                self.description = description;
            }

            GraphProgressGroupMsg::AddProgressRow {
                name,
                title,
                description
            } => {
                if self.progress_rows.contains_key(&name) {
                    return;
                }

                let index = self.progress_group.guard()
                    .push_back(ProgressBarGroup {
                        title,
                        description,
                        value: ProgressBarGroupValue::None
                    });

                self.progress_rows.insert(name, ProgressRow {
                    index,
                    last_update: Instant::now(),
                    last_fraction: 0.0,
                    graph_diffs_wma: [0.0; GRAPH_WMA_SIZE]
                });
            }

            GraphProgressGroupMsg::MarkStarted { name } => {
                if let Some(progress_row) = self.progress_rows.get(&name) {
                    self.progress_group.guard().send(
                        progress_row.index.current_index(),
                        ProgressBarGroupMsg::ShowSpinner
                    );
                }
            }

            GraphProgressGroupMsg::SetProgress {
                name,
                text,
                fraction
            } => {
                if let Some(progress_row) = self.progress_rows.get_mut(&name) {
                    self.progress_group.guard().send(
                        progress_row.index.current_index(),
                        ProgressBarGroupMsg::SetProgress { text, fraction }
                    );

                    if progress_row.last_update.elapsed() > GRAPH_DIFF_INTERVAL {
                        let mut wma_delta = (fraction - progress_row.last_fraction) * GRAPH_WMA_SIZE as f64;

                        for i in 0..(GRAPH_WMA_SIZE - 1) {
                            progress_row.graph_diffs_wma[i] = progress_row.graph_diffs_wma[i + 1];

                            wma_delta += progress_row.graph_diffs_wma[i] * (i + 1) as f64;
                        }

                        progress_row.graph_diffs_wma[GRAPH_WMA_SIZE - 1] = wma_delta / GRAPH_WMA_DIVISOR;

                        self.graph.emit(GraphMsg::AddPoint(
                            (progress_row.graph_diffs_wma[GRAPH_WMA_SIZE - 1] * GRAPH_DIFF_PRECISION) as u64
                        ));

                        progress_row.last_update = Instant::now();
                        progress_row.last_fraction = fraction;
                    }
                }
            }

            GraphProgressGroupMsg::MarkFinished { name } => {
                if let Some(progress_row) = self.progress_rows.get(&name) {
                    self.progress_group.guard().send(
                        progress_row.index.current_index(),
                        ProgressBarGroupMsg::SetFinished
                    );
                }
            }

            GraphProgressGroupMsg::ClearGraph => {
                self.graph.emit(GraphMsg::Clear);
            }

            GraphProgressGroupMsg::ClearProgressRows => {
                self.progress_rows.clear();
                self.progress_group.guard().clear();
            }
        }
    }
}
