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

use relm4::prelude::*;
use adw::prelude::*;

use agl_core::tasks::sync::mpsc::channel as mpsc;

use crate::i18n;

/// Display a simple question dialog. It will show user a given message and
/// render two buttons: "agree" and "disagree" (with given labels). The first
/// button will be suggested over the second one. The function will return
/// `true` if user chose the first button, `false` for second button or if the
/// dialog was closed.
pub async fn simple_question(
    title: impl ToString,
    body: impl ToString,
    agree: impl ToString,
    disagree: impl ToString
) -> bool {
    let title = title.to_string();
    let body = body.to_string();
    let agree = agree.to_string();
    let disagree = disagree.to_string();

    let (send, mut recv) = mpsc(1);

    gtk::glib::MainContext::default().invoke(move || {
        let dialog = adw::AlertDialog::builder()
            .heading(title)
            .body(body)
            .build();

        dialog.add_responses(&[
            ("agree",    &agree),
            ("disagree", &disagree)
        ]);

        {
            let send = send.clone();

            dialog.connect_response(Some("agree"), move |dialog, _| {
                let _ = send.blocking_send(true);

                dialog.close();
            });
        }

        {
            let send = send.clone();

            dialog.connect_response(Some("disagree"), move |dialog, _| {
                let _ = send.blocking_send(false);

                dialog.close();
            });
        }

        dialog.connect_closed(move |_| {
            let _ = send.blocking_send(false);
        });

        if let Some(window) = relm4::main_adw_application().active_window() {
            dialog.present(Some(&window));
        } else {
            dialog.present(None::<&adw::Window>);
        }
    });

    recv.recv().await.unwrap_or(false)
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
