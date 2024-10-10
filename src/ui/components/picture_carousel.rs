use adw::prelude::*;
use gtk::prelude::*;

use relm4::{factory::*, prelude::*};

use super::card::CardComponent;

#[derive(Debug)]
pub struct PictureCarouselFactory {
    pub image: String,
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for PictureCarouselFactory {
    type Init = String;
    type Input = ();
    type Output = ();
    type ParentWidget = adw::Carousel;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Picture {
            set_filename: Some(&self.image),
            add_css_class: "card",
        }
    }

    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self { image: init }
    }
}

#[derive(Debug)]
pub struct PictureCarousel {
    pictures: AsyncFactoryVecDeque<PictureCarouselFactory>,
}

#[derive(Debug)]
pub enum PictureCarouselMsg {
    Update(Vec<String>),
    NavigateLeft,
    NavigateRight,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for PictureCarousel {
    type Input = PictureCarouselMsg;
    type Output = ();
    type Init = ();

    view! {
        #[root]
        gtk::Overlay {
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 16,

                model.pictures.widget() {
                    set_height_request: CardComponent::default().height,
                    set_width_request: (CardComponent::default().height as f32 * 16.0 / 9.0) as i32
                },

                adw::CarouselIndicatorLines {
                    set_carousel: Some(&model.pictures.widget()),
                }
            },
            
            add_overlay = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_valign: gtk::Align::Center,
                set_hexpand: true,
                set_margin_all: 16,

                gtk::Button {
                    set_icon_name: "go-previous-symbolic",
                    add_css_class: "osd",
                    set_halign: gtk::Align::Start,
                    connect_clicked => PictureCarouselMsg::NavigateLeft,
                },

                gtk::Box {
                    set_hexpand: true,
                },

                gtk::Button {
                    set_icon_name: "go-next-symbolic",
                    add_css_class: "osd",
                    set_halign: gtk::Align::End,
                    connect_clicked => PictureCarouselMsg::NavigateRight,
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
            pictures: AsyncFactoryVecDeque::builder().launch_default().detach(),
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        let page_count = self.pictures.widget().n_pages();
        let current_position = self.pictures.widget().position() as u32;

        match msg {
            PictureCarouselMsg::Update(images) => {
                // Empty the vec
                self.pictures.guard().clear();

                // Fill with new images
                for image in images {
                    self.pictures.guard().push_back(image);
                }
            }
            PictureCarouselMsg::NavigateLeft => {
                if page_count != 0 {
                    let target_page = if current_position == 0 {
                        page_count - 1
                    } else {
                        current_position - 1
                    };
                    self.pictures
                        .widget()
                        .scroll_to(&self.pictures.widget().nth_page(target_page), true);
                }
            }
            PictureCarouselMsg::NavigateRight => {
                if page_count != 0 {
                    let target_page = if current_position + 1 < page_count {
                        current_position + 1
                    } else {
                        0
                    };
                    self.pictures
                        .widget()
                        .scroll_to(&self.pictures.widget().nth_page(target_page), true);
                }
            }
        }
    }
}
