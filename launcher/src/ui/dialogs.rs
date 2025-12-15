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

use std::backtrace::Backtrace;

use relm4::prelude::*;
use adw::prelude::*;

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
            ("close", "Close"),
            ("backtrace", "Backtrace")
        ]);

        dialog.set_response_appearance("close", adw::ResponseAppearance::Destructive);

        dialog.connect_response(Some("close"), |_, _| {
            relm4::main_adw_application().quit();
        });

        let backtrace_parent = dialog.clone();

        dialog.connect_response(Some("backtrace"), move |_, _| {
            relm4::view! {
                dialog = adw::Dialog {
                    set_title: "Backtrace",
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
            dialog.present(None as Option<&adw::Window>);
        }
    });
}
