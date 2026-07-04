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
                "@NelloKudo https://github.com/NelloKudo"
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
                    <li>Games manifests got optional "manifest.game.name" field</li>
                    <li>Added "free-to-play" and "adult-content" game manifest tags</li>
                    <li>Launcher will now show unstandard game manifest tags on the store page</li>
                    <li>Added Secrets API as universal solution to keep secret information that persists between module updates</li>
                    <li>Runtime APIs now accept list of paths that they cannot access. Secrets API database is automatically added to this list</li>
                    <li>Added "runtime.private_paths" launcher config property to configure list of paths to which runtime APIs will never have access</li>
                    <li>Added "TRACE" request method to HTTP API</li>
                    <li>More operations in the launcher were replaced by async alternatives</li>
                    <li>Added "base32/nix" text encoding to "str.encode" and "str.decode" APIs</li>
                </ul>

                <p>Fixed</p>

                <ul>
                    <li>Fixed launcher settings UI updating after "reactivity = none" entries changes</li>
                    <li>Fixed launcher's garbage collector deleting games integrations resources</li>
                </ul>

                <p>Changed</p>

                <ul>
                    <li>Renamed some game manifest tags, old variants kept as aliases</li>
                    <li>Game tags on the launcher's store page are now sorted</li>
                    <li>"torrent.add" API's "output_folder" option was renamed to "output_directory". Old variant is kept as alias for backward compatibility</li>
                    <li>"portal.open_folder" API was renamed to "portal.open_directory". Old function is kept as alias for backward compatibility</li>
                    <li>Modules allow lists were renamed to scopes lists. Launcher will automatically rename the standard game-integrations URL to the new one</li>
                    <li>Reduced Filesystem API's files RAM buffer size from 4 MiB to 64 KiB</li>
                    <li>Some Filesystem API functions now make symlinks only if target of family is unix</li>
                    <li>"fs.read_dir" API now uses real async operations instead of blocking function</li>
                    <li>Now launcher will silently ignore games registries and games manifests updating during startup</li>
                    <li>Games cards on the store page are now loaded in the order of their appearance in the games registries</li>
                    <li>Games lock files are now stored under "[name_hash]-[sanitized_name]" file name. Old file names will automatically be renamed to the new format if possible</li>
                    <li>Launcher will load old game packages, but show a message about game package being outdated</li>
                    <li>Changed packages hash format to "nix_base32(blake3_128([...]))"</li>
                </ul>

                <p>Removed</p>

                <ul>
                    <li>Removed some game manifest tags</li>
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
