use relm4::prelude::*;
use relm4::component::*;

use adw::prelude::*;

use crate::windows::main::MainAppMsg;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameCardVariant {
    Genshin,
    Honkai,
    StarRail
}

impl GameCardVariant {
    pub fn get_image(&self) -> &'static str {
        match self {
            Self::Genshin  => "images/genshin-cropped.jpg",
            Self::Honkai   => "images/honkai-cropped.jpg",
            Self::StarRail => "images/star-rail-cropped.jpg"
        }
    }

    pub fn get_title(&self) -> &'static str {
        match self {
            Self::Genshin  => "Genshin Impact",
            Self::Honkai   => "Honkai Impact",
            Self::StarRail => "Honkai: Star Rail"
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameCardComponent {
    pub width: i32,
    pub height: i32,
    pub variant: GameCardVariant,
    pub installed: bool
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameCardComponentMsg {
    SetVariant(GameCardVariant),
    SetWidth(i32),
    SetHeight(i32),
    SetInstalled(bool)
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for GameCardComponent {
    type Init = GameCardVariant;
    type Input = GameCardComponentMsg;
    type Output = ();

    view! {
        #[root]
        adw::Clamp {
            #[watch]
            set_maximum_size: model.width,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Overlay {
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
                        set_filename: Some(model.variant.get_image()),

                        set_content_fit: gtk::ContentFit::Cover
                    },

                    // add_overlay = &gtk::Button {
                    //     set_icon_name: "media-playback-start-symbolic",
                    //     add_css_class: "flat",

                    // }
                },

                gtk::Label {
                    set_margin_all: 12,

                    #[watch]
                    set_label: model.variant.get_title()
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            width: 260,
            height: 364, // 10:14
            variant: init,
            installed: true
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            GameCardComponentMsg::SetVariant(variant) => self.variant = variant,
            GameCardComponentMsg::SetWidth(width) => self.width = width,
            GameCardComponentMsg::SetHeight(height) => self.height = height,
            GameCardComponentMsg::SetInstalled(installed) => self.installed = installed
        }
    }
}

pub struct GameCardFactory {
    pub component: AsyncController<GameCardComponent>
}

#[relm4::factory(pub)]
impl FactoryComponent for GameCardFactory {
    type Init = GameCardVariant;
    type Input = GameCardComponentMsg;
    type Output = GameCardComponentMsg;
    type CommandOutput = ();
    type ParentInput = MainAppMsg;
    type ParentWidget = gtk::FlowBox;

    view! {
        root = gtk::Box {
            self.component.widget(),
        }
    }

    fn forward_to_parent(output: Self::Output) -> Option<MainAppMsg> {
        None
    }

    #[inline]
    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            component: GameCardComponent::builder()
                .launch(init)
                .detach()
        }
    }

    #[inline]
    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        self.component.emit(msg);
    }
}
