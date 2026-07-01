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
                "Japenese — @miyako2 https://github.com/miyako2",
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
                    <li>Games API got new values format for "enum" and "selector" settings entries.</li>
                    <li>Added "accessed_at" field to the "fs.metadata" runtime API.</li>
                    <li>Added "import" function to the runtime API.</li>
                    <li>Added Japanese and Portuguese translations.</li>
                    <li>Cache, resources, modules, and temporary directories got garbage collection mechanism controlled by related config file settings.</li>
                    <li>Launcher now will set parent game binary's directory as the spawned game process's current working directory.</li>
                </ul>

                <p>Changed</p>

                <ul>
                    <li>All network connections now respect system proxy settings. Used proxy address can be overridden in the launcher's config file.</li>
                    <li>"fs.metadata" runtime API will set "created_at", "modified_at" and "accessed_at" as "nil" instead of returning an error if user file system doesn't support these fields.</li>
                    <li>Many internal IO operations were made async.</li>
                    <li>"general.network.proxy.url" and "general.network.proxy.mode" launcher settings were replaced by "general.network.proxy" string.</li>
                    <li>Network requests user agent string was changed to "anime-games-launcher/v[version]".</li>
                    <li>Some environment variables were renamed: "LAUNCHER_DATA_FOLDER" -> "LAUNCHER_DATA_DIR"; "LAUNCHER_CONFIG_FOLDER" -> "LAUNCHER_CONFIG_DIR"; "LAUNCHER_CACHE_FOLDER" -> "LAUNCHER_CACHE_DIR".</li>
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
