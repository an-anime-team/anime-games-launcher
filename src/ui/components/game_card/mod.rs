use relm4::prelude::*;
use relm4::component::*;

use gtk::prelude::*;

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct GameCardInfo {
    pub name: String,
    pub title: String,
    pub developer: String
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameCardComponent {
    pub info: GameCardInfo,

    pub width: i32,
    pub height: i32,

    pub installed: bool,
    pub clickable: bool,
    pub display_title: bool
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCardComponentInput {
    SetInfo(GameCardInfo),

    SetWidth(i32),
    SetHeight(i32),

    SetInstalled(bool),
    SetClickable(bool),
    SetDisplayTitle(bool),

    EmitCardClick
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameCardComponentOutput {
    CardClicked {
        info: GameCardInfo,
        installed: bool
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameCardComponent {
    type Init = GameCardInfo;
    type Input = GameCardComponentInput;
    type Output = GameCardComponentOutput;

    view! {
        #[root]
        adw::Clamp {
            #[watch]
            set_maximum_size: model.width,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Overlay {
                    #[watch]
                    set_tooltip: &model.info.title,

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

                        // #[watch]
                        // set_resource: Some(&model.variant.get_image()),

                        set_content_fit: gtk::ContentFit::Cover
                    },

                    add_overlay = &gtk::Button {
                        add_css_class: "flat",

                        #[watch]
                        set_visible: model.clickable,

                        connect_clicked => GameCardComponentInput::EmitCardClick

                        // #[watch]
                        // set_icon_name: if model.installed {
                        //     "media-playback-start-symbolic"
                        // } else {
                        //     "folder-download-symbolic"
                        // }
                    }
                },

                gtk::Label {
                    set_margin_all: 12,

                    #[watch]
                    set_visible: model.display_title,

                    #[watch]
                    set_label: &model.info.title
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
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
            GameCardComponentInput::SetInfo(info)          => self.info = info,
            GameCardComponentInput::SetWidth(width)                 => self.width = width,
            GameCardComponentInput::SetHeight(height)               => self.height = height,
            GameCardComponentInput::SetInstalled(installed)        => self.installed = installed,
            GameCardComponentInput::SetClickable(clickable)        => self.clickable = clickable,
            GameCardComponentInput::SetDisplayTitle(display_title) => self.display_title = display_title,

            GameCardComponentInput::EmitCardClick => {
                sender.output(GameCardComponentOutput::CardClicked {
                    info: self.info.clone(),
                    installed: self.installed
                }).unwrap()
            }
        }
    }
}
