use relm4::prelude::*;
use gtk::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CardInfo {
    Game {
        name: String,
        title: String,
        developer: String,
        edition: String,
        picture_uri: String
    },
    Component {
        name: String,
        title: String,
        developer: String
    }
}

impl Default for CardInfo {
    #[inline]
    fn default() -> Self {
        Self::Component {
            name: String::new(),
            title: String::new(),
            developer: String::new()
        }
    }
}

impl CardInfo {
    #[inline]
    pub fn get_name(&self) -> &str {
        match self {
            Self::Game { name, .. } => name,
            Self::Component { name, .. } => name
        }
    }

    #[inline]
    pub fn get_title(&self) -> &str {
        match self {
            Self::Game { title, .. } => title,
            Self::Component { title, .. } => title
        }
    }

    #[inline]
    pub fn get_developer(&self) -> &str {
        match self {
            Self::Game { developer, .. } => developer,
            Self::Component { developer, .. } => developer
        }
    }

    #[inline]
    pub fn get_edition(&self) -> &str {
        match self {
            Self::Game { edition, .. } => edition,
            Self::Component { .. } => ""
        }
    }

    #[inline]
    pub fn get_picture_uri(&self) -> &str {
        match self {
            Self::Game { picture_uri, .. } => picture_uri,
            Self::Component { .. } => "/moe/launcher/anime-games-launcher/images/component.png"
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardComponent {
    pub info: CardInfo,

    pub width: i32,
    pub height: i32,

    pub installed: bool,
    pub clickable: bool,
    pub display_title: bool
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardComponentInput {
    SetInfo(CardInfo),

    SetWidth(i32),
    SetHeight(i32),

    SetInstalled(bool),
    SetClickable(bool),
    SetDisplayTitle(bool),

    EmitCardClick
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardComponentOutput {
    CardClicked {
        info: CardInfo,
        installed: bool
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for CardComponent {
    type Init = CardInfo;
    type Input = CardComponentInput;
    type Output = CardComponentOutput;

    view! {
        #[root]
        adw::Clamp {
            #[watch]
            set_maximum_size: model.width,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Overlay {
                    #[watch]
                    set_tooltip: model.info.get_title(),

                    gtk::Picture {
                        set_valign: gtk::Align::Start,
                        set_halign: gtk::Align::Start,

                        #[watch]
                        set_width_request: model.width,

                        #[watch]
                        set_height_request: model.height,

                        #[watch]
                        set_opacity: if model.installed {
                            1.0
                        } else {
                            0.4
                        },

                        add_css_class: "card",
                        add_css_class: "game-card",

                        // #[watch]
                        // set_css_classes: if model.installed {
                        //     &["card", "game-card"]
                        // } else {
                        //     &["card", "game-card", "game-card--not-installed"]
                        // },

                        #[watch]
                        set_filename: Some(model.info.get_picture_uri()),

                        set_content_fit: gtk::ContentFit::Cover
                    },

                    add_overlay = &gtk::Button {
                        add_css_class: "flat",

                        #[watch]
                        set_visible: model.clickable,

                        connect_clicked => CardComponentInput::EmitCardClick

                        // #[watch]
                        // set_icon_name: if model.installed {
                        //     "media-playback-start-symbolic"
                        // } else {
                        //     "folder-download-symbolic"
                        // }
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,

                    set_margin_all: 12,

                    #[watch]
                    set_visible: model.display_title,

                    gtk::Label {
                        #[watch]
                        set_label: model.info.get_title()
                    },

                    gtk::Label {
                        #[watch]
                        set_markup: &format!("  <span foreground=\"grey\">({})</span>", model.info.get_edition())
                    }
                }
            }
        }
    }

    async fn init(init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            info: init,

            // 10:14
            width: 240, // 260,
            height: 336, // 364,

            installed: true,
            clickable: true,
            display_title: true
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            CardComponentInput::SetInfo(info)              => self.info          = info,
            CardComponentInput::SetWidth(width)                 => self.width         = width,
            CardComponentInput::SetHeight(height)               => self.height        = height,
            CardComponentInput::SetInstalled(installed)        => self.installed     = installed,
            CardComponentInput::SetClickable(clickable)        => self.clickable     = clickable,
            CardComponentInput::SetDisplayTitle(display_title) => self.display_title = display_title,

            CardComponentInput::EmitCardClick => {
                sender.output(CardComponentOutput::CardClicked {
                    info: self.info.clone(),
                    installed: self.installed
                }).unwrap()
            }
        }
    }
}
