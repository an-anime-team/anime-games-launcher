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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AboutWindow;

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for AboutWindow {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        #[root]
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
                "Italian — Martino Papero https://github.com/martymarty004",
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
            set_release_notes: r#"
                <p>Added</p>

                <ul>
                    "<li>Added Italian translations</li>
                    "<li>Added game integration agreements</li>
                    "<li>When launched in debug mode ('--agl-debug' flag) the launcher will show entries names for settings and components as hints</li>
                    "<li>Added new settings entry format for secrets (e.g. passwords)</li>
                    "<li>Added optional game components system which required a big rewrite of the launcher's backend code</li>
                    "<li>Added Protobuf API to the lua runtime</li>
                </ul>

                <p>Fixed</p>

                <ul>
                    <li>Selectable strings are not highlighted, focused or selected by default anymore</li>
                    <li>Game variants are now properly selected and propagated to the lua side</li>
                    <li>Actions in the actions pipeline window now properly render their descriptions</li>
                    <li>Fixed a bug when the game library details window was updated 3 times instead of 1</li>
                    <li>Fixed automatic game selection on the library page if "open in library" button on the game store page was clicked</li>
                    <li>Fixed system language identification</li>
                </ul>

                <p>Changed</p>

                <ul>
                    <li>HTTP API headers are now case-insensitive</li>
                </ul>
            "#
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
