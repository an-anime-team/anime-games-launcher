use adw::prelude::*;
use relm4::prelude::*;

use crate::prelude::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardSize {
    #[default]
    Large,

    Medium,
    Small
}

impl CardSize {
    #[inline]
    pub fn width(&self) -> i32 {
        match self {
            Self::Large  => 240,
            Self::Medium => 160,
            Self::Small  => 40
        }
    }

    #[inline]
    pub fn height(&self) -> i32 {
        // 10:14
        match self {
            Self::Large  => 336,
            Self::Medium => 224,
            Self::Small  => 56
        }
    }

    #[inline]
    pub fn size(&self) -> (i32, i32) {
        (self.width(), self.height())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CardComponentInput {
    SetImage(Option<ImagePath>),
    SetTitle(Option<String>),

    SetSize(CardSize),
    SetClickable(bool),
    SetBlurred(bool),

    EmitClick
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardComponentOutput {
    Clicked
}

#[derive(Debug)]
pub struct CardComponent {
    picture: AsyncController<LazyPictureComponent>,

    pub size: CardSize,
    pub title: Option<String>,
    pub clickable: bool
}

impl CardComponent {
    pub fn large() -> Self {
        let size = CardSize::Large;

        Self {
            picture: LazyPictureComponent::builder()
                .launch(LazyPictureComponent {
                    image: None,

                    width: Some(size.width()),
                    height: Some(size.height()),

                    blurred: false
                })
                .detach(),

            size,
            title: None,
            clickable: false
        }
    }

    pub fn medium() -> Self {
        let size = CardSize::Medium;

        Self {
            picture: LazyPictureComponent::builder()
                .launch(LazyPictureComponent {
                    image: None,

                    width: Some(size.width()),
                    height: Some(size.height()),

                    blurred: false
                })
                .detach(),

            size,
            title: None,
            clickable: false
        }
    }

    pub fn small() -> Self {
        let size = CardSize::Small;

        Self {
            picture: LazyPictureComponent::builder()
                .launch(LazyPictureComponent {
                    image: None,

                    width: Some(size.width()),
                    height: Some(size.height()),

                    blurred: false
                })
                .detach(),

            size,
            title: None,
            clickable: false
        }
    }

    pub fn with_image(self, image: ImagePath) -> Self {
        self.picture.emit(LazyPictureComponentMsg::SetImage(Some(image)));

        self
    }

    #[inline]
    pub fn with_title(mut self, title: impl ToString) -> Self {
        self.title = Some(title.to_string());

        self
    }

    #[inline]
    pub fn with_clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;

        self
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for CardComponent {
    type Init = Self;
    type Input = CardComponentInput;
    type Output = CardComponentOutput;

    view! {
        #[root]
        adw::Clamp {
            #[watch]
            set_maximum_size: model.size.width(),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::Overlay {
                    #[watch]
                    set_tooltip?: &model.title,

                    model.picture.widget() {
                        add_css_class: "card"
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

    #[inline]
    async fn init(model: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            CardComponentInput::SetTitle(title) => self.title = title,

            CardComponentInput::SetImage(image) => {
                self.picture.emit(LazyPictureComponentMsg::SetImage(image));
            }

            CardComponentInput::SetSize(size) => {
                self.size = size;

                self.picture.emit(LazyPictureComponentMsg::SetWidth(Some(size.width())));
                self.picture.emit(LazyPictureComponentMsg::SetHeight(Some(size.height())));
            }

            CardComponentInput::SetClickable(clickable) => self.clickable = clickable,

            CardComponentInput::SetBlurred(blurred) => {
                self.picture.emit(LazyPictureComponentMsg::SetBlurred(blurred));
            }

            CardComponentInput::EmitClick => {
                let _ = sender.output(CardComponentOutput::Clicked);
            }
        }
    }
}
