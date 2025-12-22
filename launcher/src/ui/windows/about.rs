use gtk::prelude::WidgetExt;
use relm4::prelude::*;

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
                "Русский, English — Nikita Podvirnyi https://github.com/krypt0nn"
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
                    "<li>Added separate read and write permissions to sandboxed filesystem paths in modules runtime</li>",
                    "<li>Added modules allow lists. Modules runtime tries to read module's scope from it and falls back to default values</li>",
                    "<li>Add module scope to the game package lock. This scope will be applied to all the modules used by the game integration (game-specific sandbox permissions)</li>",
                    "<li>Added portal API</li>",
                    "<li>Added logging for runtime modules loading</li>",
                "</ul>",

                "<p>Fixed</p>",

                "<ul>",
                    "<li>Fixed layout of the games store details page</li>",
                    "<li>Provide most of default lua functions for runtime modules</li>",
                    "<li>Input resources of a package are now allowed to be read by output modules of this package</li>",
                    "<li>Fixed panic message on application close</li>",
                    "<li>Fixed game launch info hint being 'nil' when unset</li>",
                "</ul>",

                "<p>Changed</p>",

                "<ul>",
                    "<li>Changed logging filters for stdout and 'debug.log' file</li>",
                    "<li>Game integration pipeline actions now don't need to return any (boolean) output from `perform` functions</li>",
                    "<li>Changed pipeline actions graph update rate to 0.5 seconds</li>",
                    "<li>In many manifests 'format' is expected instead of 'version'. For now 'version' is accepted as fallback field</li>",
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
