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

use std::backtrace::Backtrace;
use std::sync::{Arc, Mutex};

use relm4::prelude::*;
use adw::prelude::*;

use crate::i18n;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogActionAppearance {
    #[default]
    Normal,
    Suggested,
    Destructive
}

impl From<DialogActionAppearance> for adw::ResponseAppearance {
    fn from(value: DialogActionAppearance) -> Self {
        match value {
            DialogActionAppearance::Normal      => Self::Default,
            DialogActionAppearance::Suggested   => Self::Suggested,
            DialogActionAppearance::Destructive => Self::Destructive
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DialogAction {
    pub name: String,
    pub title: String,
    pub appearance: DialogActionAppearance
}

impl DialogAction {
    pub fn new(name: impl ToString, title: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            title: title.to_string(),
            appearance: DialogActionAppearance::default()
        }
    }

    #[inline]
    pub const fn with_appearance(
        mut self,
        appearance: DialogActionAppearance
    ) -> Self {
        self.appearance = appearance;

        self
    }

    #[inline]
    pub const fn as_suggested(mut self) -> Self {
        self.appearance = DialogActionAppearance::Suggested;

        self
    }

    #[inline]
    pub const fn as_destructive(mut self) -> Self {
        self.appearance = DialogActionAppearance::Destructive;

        self
    }
}

/// Present a dialog with given action buttons. Return name of selected action,
/// or `None` if dialog was closed without choosing anything.
///
/// Note: this is a blocking function. You likely want to run it from a
/// different thread.
pub fn present(
    title: impl ToString,
    body: impl ToString,
    actions: impl IntoIterator<Item = DialogAction>
) -> Option<String> {
    let title = title.to_string();
    let body = body.to_string();

    // Unfortunately a normal solution with async function and 1-size mpsc
    // channel doesn't work here, or at least I couldn't make it to work. The
    // problem is that the dialog is being closed *before* the message is
    // received on the other side, so it gets disappeared with the dialog,
    // and this function returns default value - `None`. So I had to make this
    // hacky mess with blocking function and a shared mutex.
    let response = Arc::new(Mutex::new(None));

    let actions = actions.into_iter()
        .map(|action| (action.name, action.title, action.appearance.into()))
        .collect::<Box<[_]>>();

    {
        let response = response.clone();

        gtk::glib::MainContext::default().invoke(move || {
            let dialog = adw::AlertDialog::builder()
                .heading(title)
                .body(body)
                .build();

            for (name, title, appearance) in actions {
                let response = response.clone();

                dialog.add_response(&name, &title);
                dialog.set_response_appearance(&name, appearance);

                dialog.connect_response(Some(&name.clone()), move |dialog, _| {
                    if let Ok(mut lock) = response.lock() {
                        *lock = Some(Some(name.clone()));
                    }

                    dialog.close();
                });
            }

            // dialog.connect_closed(move |_| {
            //     if let Ok(mut lock) = response.lock()
            //         && lock.is_none()
            //     {
            //         *lock = Some(None);
            //     }
            // });

            if let Some(window) = relm4::main_adw_application().active_window() {
                dialog.present(Some(&window));
            } else {
                dialog.present(None::<&adw::Window>);
            }
        });
    }

    while Arc::strong_count(&response) > 1 {
        if let Some(value) = response.lock().ok()?.take() {
            return value;
        }
    }

    response.lock()
        .ok()?
        .take()
        .flatten()
}

/// Display error dialog. It will allow user to look through the given error,
/// current thread's backtrace, and close the dialog to continue working with
/// app.
pub fn error(title: impl ToString, body: impl ToString) {
    let backtrace = Backtrace::force_capture().to_string();

    let title = title.to_string();
    let body = body.to_string();

    gtk::glib::MainContext::default().invoke(move || {
        let dialog = adw::AlertDialog::builder()
            .heading(title)
            .body(body)
            .build();

        dialog.add_responses(&[
            ("close",     i18n!("close").unwrap_or("Close")),
            ("backtrace", i18n!("backtrace").unwrap_or("Backtrace"))
        ]);

        dialog.connect_response(Some("close"), |dialog, _| {
            dialog.close();
        });

        let backtrace_parent = dialog.clone();

        dialog.connect_response(Some("backtrace"), move |_, _| {
            relm4::view! {
                dialog = adw::Dialog {
                    set_title: i18n!("backtrace").unwrap_or("Backtrace"),
                    set_size_request: (900, 700),

                    #[wrap(Some)]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        adw::HeaderBar,

                        gtk::ScrolledWindow {
                            set_hexpand: true,
                            set_vexpand: true,

                            gtk::Label {
                                set_halign: gtk::Align::Start,

                                set_selectable: true,
                                set_focusable: false,

                                set_label: backtrace.as_str()
                            }
                        }
                    }
                }
            }

            dialog.present(Some(&backtrace_parent));
        });

        if let Some(window) = relm4::main_adw_application().active_window() {
            dialog.present(Some(&window));
        } else {
            dialog.present(None::<&adw::Window>);
        }
    });
}

/// Display critical error dialog. It will allow user to look through the given
/// error, current thread's backtrace, and close the app safely.
pub fn critical_error(title: impl ToString, body: impl ToString) {
    let backtrace = Backtrace::force_capture().to_string();

    let title = title.to_string();
    let body = body.to_string();

    gtk::glib::MainContext::default().invoke(move || {
        let dialog = adw::AlertDialog::builder()
            .heading(title)
            .body(body)
            .can_close(false)
            .build();

        dialog.add_responses(&[
            ("close",     i18n!("close").unwrap_or("Close")),
            ("backtrace", i18n!("backtrace").unwrap_or("Backtrace"))
        ]);

        dialog.set_response_appearance("close", adw::ResponseAppearance::Destructive);

        dialog.connect_response(Some("close"), |_, _| {
            relm4::main_adw_application().quit();
        });

        let backtrace_parent = dialog.clone();

        dialog.connect_response(Some("backtrace"), move |_, _| {
            relm4::view! {
                dialog = adw::Dialog {
                    set_title: i18n!("backtrace").unwrap_or("Backtrace"),
                    set_size_request: (900, 700),

                    #[wrap(Some)]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        adw::HeaderBar,

                        gtk::ScrolledWindow {
                            set_hexpand: true,
                            set_vexpand: true,

                            gtk::Label {
                                set_halign: gtk::Align::Start,

                                set_selectable: true,
                                set_focusable: false,

                                set_label: backtrace.as_str()
                            }
                        }
                    }
                }
            }

            dialog.present(Some(&backtrace_parent));
        });

        if let Some(window) = relm4::main_adw_application().active_window() {
            dialog.present(Some(&window));
        } else {
            dialog.present(None::<&adw::Window>);
        }
    });
}
