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
            set_filename: Some(&self.image)
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
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for PictureCarousel {
    type Input = PictureCarouselMsg;
    type Output = ();
    type Init = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 16,

            #[local_ref]
            pictures -> adw::Carousel {
                set_height_request: CardComponent::default().height,
            },

            adw::CarouselIndicatorLines {
                set_carousel: Some(pictures),
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

        let pictures = model.pictures.widget();

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            PictureCarouselMsg::Update(images) => {
                // Empty the vec
                self.pictures.guard().clear();

                // Fill with new images
                for image in images {
                    self.pictures.guard().push_back(image);
                }
            }
        }
    }
}
