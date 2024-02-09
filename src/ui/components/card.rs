use gtk::prelude::*;
use relm4::prelude::*;

// 10:14
pub const DEFAULT_SIZE: (i32, i32) = (240, 336);
pub const MEDIUM_SIZE: (i32, i32)  = (160, 224);
pub const SMALL_SIZE: (i32, i32)   = (40, 56);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardComponentInput {
    SetImage(Option<String>),
    SetTitle(Option<String>),

    SetWidth(i32),
    SetHeight(i32),

    SetClickable(bool),
    SetBlurred(bool),

    EmitClick
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardComponentOutput {
    Clicked
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardComponent {
    pub image: Option<String>,
    pub title: Option<String>,

    pub width: i32,
    pub height: i32,

    pub clickable: bool,
    pub blurred: bool
}

impl Default for CardComponent {
    #[inline]
    fn default() -> Self {
        Self {
            image: None,
            title: None,

            width: DEFAULT_SIZE.0,
            height: DEFAULT_SIZE.1,

            clickable: false,
            blurred: false
        }
    }
}

impl CardComponent {
    #[inline]
    pub fn medium() -> Self {
        Self {
            width: MEDIUM_SIZE.0,
            height: MEDIUM_SIZE.1,

            ..Self::default()
        }
    }

    #[inline]
    pub fn small() -> Self {
        Self {
            width: SMALL_SIZE.0,
            height: SMALL_SIZE.1,

            ..Self::default()
        }
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for CardComponent {
    type Init = CardComponent;
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
                    set_tooltip?: &model.title,

                    gtk::Picture {
                        set_valign: gtk::Align::Start,
                        set_halign: gtk::Align::Start,

                        set_content_fit: gtk::ContentFit::Cover,

                        add_css_class: "card",

                        #[watch]
                        set_size_request: (model.width, model.height),

                        #[watch]
                        set_opacity: if model.blurred { 0.4 } else { 1.0 },

                        #[watch]
                        set_resource?: model.image.as_ref()
                            .and_then(|image| image
                                .starts_with(crate::APP_RESOURCE_PREFIX)
                                .then_some(Some(image.as_str()))),

                        #[watch]
                        set_filename?: model.image.as_ref()
                            .and_then(|image| (!image
                                .starts_with(crate::APP_RESOURCE_PREFIX))
                                .then_some(Some(image.as_str())))
                    },

                    add_overlay = &gtk::Button {
                        add_css_class: "flat",

                        #[watch]
                        set_visible: model.clickable,

                        connect_clicked => CardComponentInput::EmitClick
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Center,

                    set_margin_all: 12,

                    #[watch]
                    set_visible: model.title.is_some(),

                    gtk::Label {
                        #[watch]
                        set_label?: &model.title
                    }
                }
            }
        }
    }

    async fn init(model: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            CardComponentInput::SetTitle(title) => self.title = title,
            CardComponentInput::SetImage(image) => self.image = image,

            CardComponentInput::SetWidth(width)   => self.width  = width,
            CardComponentInput::SetHeight(height) => self.height = height,

            CardComponentInput::SetClickable(clickable) => self.clickable = clickable,
            CardComponentInput::SetBlurred(blurred)     => self.blurred   = blurred,

            CardComponentInput::EmitClick => {
                sender.output(CardComponentOutput::Clicked).unwrap()
            }
        }
    }
}
