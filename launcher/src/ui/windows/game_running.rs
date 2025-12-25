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

use std::process::Child;
use std::time::{Instant, Duration};

use relm4::prelude::*;
use adw::prelude::*;

use agl_core::tasks;
use agl_core::export::tasks::tokio;

use crate::consts;
use crate::utils;
use crate::i18n;
use crate::ui::dialogs;

const UPDATE_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug)]
pub enum GameRunningWindowMsg {
    SetChild {
        game_title: String,
        child: Child
    },

    Update,
    Kill,
    Close
}

#[derive(Debug)]
pub struct GameRunningWindow {
    window: Option<adw::Dialog>,

    game_title: Option<String>,

    child: Option<Child>,

    running_since: Option<Instant>,
    running_time: Option<String>,
    running_handle: Option<tasks::JoinHandle<()>>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameRunningWindow {
    type Init = ();
    type Input = GameRunningWindowMsg;
    type Output = ();

    view! {
        #[root]
        _window = adw::Dialog {
            set_size_request: (400, 260),
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
                        adw::ActionRow {
                            set_title: i18n!("game_process_id")
                                .unwrap_or("Game process"),

                            add_suffix = &gtk::Label {
                                set_selectable: true,

                                #[watch]
                                set_label: model.child.as_ref()
                                    .map(|child| child.id().to_string())
                                    .as_deref()
                                    .unwrap_or_default()
                            }
                        },

                        adw::ActionRow {
                            set_title: i18n!("game_running_for")
                                .unwrap_or("Running for"),

                            add_suffix = &gtk::Label {
                                #[watch]
                                set_label: model.running_time.as_deref()
                                    .unwrap_or_default()
                            }
                        }
                    },

                    adw::PreferencesGroup {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,

                            gtk::Button {
                                add_css_class: "pill",
                                add_css_class: "destructive-action",

                                adw::ButtonContent {
                                    set_label: i18n!("kill_game_process")
                                        .unwrap_or("Kill"),

                                    set_icon_name: "violence-symbolic"
                                },

                                connect_clicked => GameRunningWindowMsg::Kill
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
        let mut model = Self {
            window: None,
            game_title: None,
            child: None,
            running_since: None,
            running_time: None,
            running_handle: None
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
            GameRunningWindowMsg::SetChild { game_title, child } => {
                self.game_title = Some(game_title);
                self.child = Some(child);

                self.running_since = Some(Instant::now());

                self.running_handle = Some(tasks::spawn(async move {
                    loop {
                        sender.input(GameRunningWindowMsg::Update);

                        tokio::time::sleep(UPDATE_INTERVAL).await;
                    }
                }));
            }

            GameRunningWindowMsg::Update => {
                if let Some(running_since) = &self.running_since {
                    let running_time = running_since.elapsed().as_secs();

                    self.running_time = Some(utils::pretty_seconds(running_time));
                }

                if let Some(child) = &mut self.child
                    && matches!(child.try_wait(), Ok(Some(_)))
                {
                    sender.input(GameRunningWindowMsg::Kill);
                }
            }

            GameRunningWindowMsg::Kill => {
                self.running_since = None;
                self.running_time = None;

                #[allow(clippy::collapsible_if)]
                if let Some(mut child) = self.child.take() {
                    if let Err(err) = child.kill() {
                        tracing::error!(?err, "failed to kill running game process");

                        dialogs::error(
                            i18n!("failed_kill_game_process")
                                .unwrap_or("Failed to kill running game process"),
                            err.to_string()
                        );
                    }
                }

                if let Some(handle) = self.running_handle.take() {
                    handle.abort();
                }

                sender.input(GameRunningWindowMsg::Close);
            }

            GameRunningWindowMsg::Close => {
                if let Some(window) = &self.window {
                    window.force_close();
                }
            }
        }
    }
}
