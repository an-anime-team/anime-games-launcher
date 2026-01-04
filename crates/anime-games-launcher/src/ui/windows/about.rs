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

use relm4::prelude::*;
use adw::prelude::*;

use crate::consts;

lazy_static::lazy_static! {
    static ref APP_VERSION: String = if *consts::APP_DEBUG && !consts::APP_VERSION.contains('-') {
        format!("{}-dev", consts::APP_VERSION)
    } else {
        consts::APP_VERSION.to_string()
    };
}

#[derive(Debug, Clone, Copy)]
pub struct AboutWindow;

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for AboutWindow {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        adw::AboutDialog {
            set_application_name: "Anime Games Launcher",
            set_application_icon: consts::APP_ID,

            set_hexpand: true,

            set_website: "https://github.com/an-anime-team/anime-games-launcher",
            // set_issue_url: "https://github.com/an-anime-team/anime-games-launcher/issues",
            set_support_url: "https://discord.gg/ck37X6UWBp",

            set_license_type: gtk::License::Gpl30,
            set_version: &APP_VERSION,

            set_developer_name: "Nikita Podvirnyi",

            set_developers: &[
                "Nikita Podvirnyi https://github.com/krypt0nn"
            ],

            add_credit_section: (Some("Contributors"), &[
                "Dylan Donnell https://github.com/dy-tea",
                "@mkrsym1 https://github.com/mkrsym1"
            ]),

            add_credit_section: (Some("Linux packaging"), &[
                "Flatpak — @NelloKudo https://github.com/NelloKudo",
                "AUR — @xstraok",
                "Gentoo ebuild — @JohnTheCoolingFan https://github.com/JohnTheCoolingFan"
            ]),

            set_translator_credits: &[
                "Русский, English — Nikita Podvirnyi https://github.com/krypt0nn",
                "German — @caemputer https://github.com/caemputer",
                "Portuguese — João Dias https://github.com/retrozinndev"
            ].join("\n"),

            set_debug_info: &[
                format!("agl_core: {}", agl_core::VERSION),
                format!("agl_packages: {}", agl_packages::VERSION),
                format!("agl_runtime: {}", agl_runtime::VERSION),
                format!("agl_games: {}", agl_games::VERSION),
                String::new(),
                format!("gtk: {}.{}.{}", gtk::major_version(), gtk::minor_version(), gtk::micro_version()),
                format!("libadwaita: {}.{}.{}", adw::major_version(), adw::minor_version(), adw::micro_version()),
                format!("pango: {}", gtk::pango::version_string()),
                format!("cairo: {}", gtk::cairo::version_string())
            ].join("\n"),

            set_release_notes_version: &APP_VERSION,
            set_release_notes: &[
                "<p>Added</p>",

                "<ul>",
                    "<li>'torrent.add' API got 'restart' option to restart already added torrents</li>",
                    "<li>Added launcher localization using in-house 'agl-locale' crate; English, Russian, German and Portuguese languages are supported now</li>",
                    "<li>Game integrations now can be returned by a function to support lazy loading</li>",
                    "<li>Added task API with 'Promise' userdata type. If returned from runtime - the actual work happens in background and doesn't block lua engine thread</li>",
                    "<li>Added 'await' runtime function to resolve different lua types, including functions, coroutines (threads), 'Promises' and more</li>",
                    "<li>Added 'Bytes' userdata type to replace tables of numbers used to represent bytes on lua side. Most of runtime API methods were reworked to return and accept this custom type</li>",
                    "<li>Added system API to query system-related information, currently local and UTC time, environment variables and binaries paths</li>",
                "</ul>",

                "<p>Fixed</p>",

                "<ul>",
                    "<li>Actions pipeline execution graph now resets on window close</li>",
                    "<li>Fixed vertical distance between store page game cards</li>",
                    "<li>Fixed 'http.fetch' options parsing</li>",
                    "<li>Process API now doesn't resolve the binary path and doesn't check for relative path</li>",
                "</ul>",

                "<p>Changed</p>",

                "<ul>",
                    "<li>Force torrent API to add global torrents list to each added torrent</li>",
                    "<li>Display progress bar in actions pipeline window even if current progress is 0</li>",
                    "<li>Updated lua engine version; 64 bit numbers should be supported now</li>",
                    "<li>Changed required GTK4 and libadwaita versions to support older linux distros</li>",
                    "<li>Add more environment variables to parse system language from</li>",
                    "<li>Renamed network API to HTTP API</li>",
                    "<li>Most of runtime API methods were promisified (reworked to return 'Promise') and perform actual work in background to not to block lua engine thread</li>",
                    "<li>In-RAM memory buffers for some APIs were increased for better performance</li>",
                    "<li>Sqlite API now can accept functions, coroutines (threads), 'Promise' and 'Bytes' types as query params (they will be resolved into actual values)</li>",
                "</ul>",

                "<p>Removed</p>",

                "<ul>",
                    "<li>Removed unused 'utils' and 'i18n' launcher modules</li>",
                "</ul>"
            ].join("\n")
        }
    }

    #[inline]
    async fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>
    ) -> AsyncComponentParts<Self> {
        let model = Self;
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }
}
