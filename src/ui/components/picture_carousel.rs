use adw::prelude::*;
use relm4::prelude::*;

use super::*;

#[derive(Debug)]
pub struct PictureCarousel {
    pictures: AsyncFactoryVecDeque<PictureCarouselFactory>
}

#[derive(Debug)]
pub enum PictureCarouselMsg {
    SetImages(Vec<ImagePath>),

    NavigateLeft,
    NavigateRight
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for PictureCarousel {
    type Init = ();
    type Input = PictureCarouselMsg;
    type Output = ();

    view! {
        #[root]
        gtk::Overlay {
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 8,

                model.pictures.widget() {
                    // set_width_request: (CardSize::Large.width() as f32 * 16.0 / 9.0) as i32,
                    set_height_request: CardSize::Large.height()
                },

                adw::CarouselIndicatorLines {
                    set_carousel: Some(model.pictures.widget()),
                }
            },

            add_overlay = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_valign: gtk::Align::Center,

                set_hexpand: true,
                set_margin_all: 16,

                gtk::Button {
                    set_halign: gtk::Align::Start,

                    add_css_class: "osd",
                    set_icon_name: "go-previous-symbolic",
                    
                    connect_clicked => PictureCarouselMsg::NavigateLeft
                },

                gtk::Box {
                    set_hexpand: true,
                },

                gtk::Button {
                    set_halign: gtk::Align::End,

                    add_css_class: "osd",
                    set_icon_name: "go-next-symbolic",

                    connect_clicked => PictureCarouselMsg::NavigateRight
                } 
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            pictures: AsyncFactoryVecDeque::builder()
                .launch_default()
                .detach()
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        let current = self.pictures.widget().position() as u32;
        let total = self.pictures.widget().n_pages();

        match msg {
            PictureCarouselMsg::SetImages(images) => {
                let mut guard = self.pictures.guard();

                guard.clear();

                for image in images {
                    guard.push_back(image);
                }
            }

            PictureCarouselMsg::NavigateLeft => {
                if total != 0 {
                    let target_page = current.checked_sub(1)
                        .unwrap_or(total - 1);

                    self.pictures.widget()
                        .scroll_to(&self.pictures.widget().nth_page(target_page), true);
                }
            }

            PictureCarouselMsg::NavigateRight => {
                if total != 0 {
                    let target_page = (current + 1) % total;

                    self.pictures.widget()
                        .scroll_to(&self.pictures.widget().nth_page(target_page), true);
                }
            }
        }
    }
}

#[derive(Debug)]
struct PictureCarouselFactory {
    picture: AsyncController<LazyPictureComponent>
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for PictureCarouselFactory {
    type Init = ImagePath;
    type Input = ();
    type Output = ();
    type ParentWidget = adw::Carousel;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            self.picture.widget() {
                set_valign: gtk::Align::Fill,

                add_css_class: "card"
            }
        }
    }

    async fn init_model(image: Self::Init, _index: &DynamicIndex, _sender: AsyncFactorySender<Self>) -> Self {
        Self {
            picture: LazyPictureComponent::builder()
                .launch(LazyPictureComponent {
                    image: Some(image),

                    ..LazyPictureComponent::default()
                })
                .detach()
        }
    }
}
