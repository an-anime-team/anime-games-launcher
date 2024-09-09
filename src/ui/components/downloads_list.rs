use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use super::prelude::CardComponent;
use crate::utils::pretty_bytes;

/// DownloadsRow will by itself display a single row for the current download
/// DownloadsListFactory will create a List of DownloadsRow for the scheduled downloads
///
/// TODO: Create controller for active download and vecdequeue for scheduled
#[derive(Debug)]
pub struct DownloadsRow {
    pub card: AsyncController<CardComponent>,
    pub title: Option<&'static str>,
    pub version: Option<&'static str>,
    pub edition: Option<&'static str>,
    pub size: Option<u64>,
    pub downloaded: Option<u64>,
    pub downloading: bool,
    index: Option<DynamicIndex>,
}

#[derive(Debug)]
pub enum DownloadsRowMsg {
    ToggleDownloading,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for DownloadsRow {
    type Init = ();
    type Input = DownloadsRowMsg;
    type Output = ();

    view! {
        #[root]
        adw::ActionRow {
            add_prefix = model.card.widget() {
                set_opacity: 0.5,
                set_halign: gtk::Align::Start,
            },

            #[watch]
            set_title?: model.title,

            #[watch]
            set_subtitle: &format!("{} ∙ {}", model.edition.unwrap_or("N/A"), model.version.unwrap_or("N/A")),

            add_suffix = &gtk::Label {
                #[watch]
                set_label: &format!("{} / {}", pretty_bytes(model.downloaded.unwrap_or(0)), pretty_bytes(model.size.unwrap_or(0))),
            },

            add_suffix = &gtk::Box {
                set_size_request: (16, 16),
            },

            add_suffix = &gtk::ProgressBar {
                set_align: gtk::Align::Center,
                #[watch]
                set_fraction: model.downloaded.unwrap_or(0) as f64 / model.size.unwrap_or(0) as f64,
            },

            add_suffix = &gtk::Box {
                set_size_request: (16, 16),
            },

            add_suffix = &gtk::Label {
                #[watch]
                set_label: &format!("{:.2}%", model.downloaded.unwrap_or(0) as f64 / model.size.unwrap_or(0) as f64 * 100.0),
            },

            add_suffix = &gtk::Box {
                set_size_request: (16, 16),
            },

            add_suffix = &gtk::Button {
                set_align: gtk::Align::Center,
                #[watch]
                set_icon_name: if model.downloading {"media-playback-pause-symbolic"} else {"media-playback-start-symbolic"},
                connect_clicked => DownloadsRowMsg::ToggleDownloading,
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            card: CardComponent::builder()
                .launch(CardComponent {
                    image: Some(String::from("")),
                    title: None,
                    ..CardComponent::small()
                })
                .detach(),
            title: Some("Genshin Impact"),
            version: Some("5.0.0"),
            edition: Some("Global"),
            size: Some(64500000000),
            downloaded: None,
            downloading: false,
            index: None,
        };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            DownloadsRowMsg::ToggleDownloading => self.downloading = !self.downloading,
        }
    }
}
