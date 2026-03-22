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
                "Portuguese — João Dias https://github.com/retrozinndev",
                "Indonesian — @yumekarisu https://github.com/yumekarisu",
                "And other contributors"
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
                    "<li>Added 'anirun' binary to test luau runtime packages and modules</li>",
                    "<li>Added BSON format support in 'string.encode', 'string.decode' runtime APIs</li>",
                    "<li>Added support for functions without output (without 'return' statements) in 'Promise.poll' runtime API</li>",
                    "<li>Added custom user agent in all the HTTP requests, including downloader runtime API: 'User-Agent: anime-games-launcher/version'</li>",
                    "<li>Added standard luau 'number' table to support advanced arithmetic functions in the luau runtime</li>",
                    "<li>Added 'stringify' runtime API to convert lua objects to strings and return them back to the lua runtime side</li>",
                    "<li>Added \"No games available\" status in store page</li>",
                    "<li>Added 'str.lowercase', 'str.uppercase' and 'str.trim' runtime APIs to support unicode characters (standard lua functions work only with ASCII)</li>",
                    "<li>Added 'selector' and 'number' game settings entry formats</li>",
                    "<li>Added search bar to `enum` game settings entry if there's 10 or more values available</li>",
                    "<li>Added `tools.get_buttons` game integration function. Now integrations can add their own buttons to the library details page for different needs</li>",
                    "<li>Added launcher-side luau runtime garbage collection task, and related 'runtime.collect_garbage_interval' config</li>",
                "</ul>",

                "<p>Fixed</p>",

                "<ul>",
                    "<li>Fixed 'hash.digitize_file' runtime API stack overflow</li>",
                    "<li>Fixed 'POST' method name in HTTP runtime API</li>",
                    "<li>Fixed game scope overwriting with default values in added games' manifests</li>",
                    "<li>Fixed default image rendering in horizontal lazy loadable pictures</li>",
                "</ul>",

                "<p>Changed</p>",

                "<ul>",
                    "<li>Disable human panic in debug builds</li>",
                    "<li>Return 'nil' in 'compression.read' runtime API if nothing to read</li>",
                    "<li>Changed 'string.encode' and 'string.decode' runtime API args order</li>",
                    "<li>In game details within the store page, carousel will now hide controls if there's only one picture available</li>",
                    "<li>Featured games are now shown before non-featured games</li>",
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
