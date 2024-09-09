use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use super::prelude::CardComponent;
use crate::utils::pretty_bytes;

#[derive(Debug)]
pub struct DownloadsRowInit {
    pub card_image: Option<String>,
    pub title: Option<&'static str>,
    pub version: Option<&'static str>,
    pub edition: Option<&'static str>,
    pub size: Option<u64>,
    pub start_download: bool,
}

impl DownloadsRowInit {
    #[inline]
    pub fn new(
        card_image: impl ToString,
        title: &'static str,
        version: &'static str,
        edition: &'static str,
        size: u64,
        start_download: bool,
    ) -> Self {
        Self {
            card_image: Some(card_image.to_string()),
            title: Some(title),
            version: Some(version),
            edition: Some(edition),
            size: Some(size),
            start_download,
        }
    }
}

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
}

#[derive(Debug)]
pub enum DownloadsRowMsg {
    SetDownloaded(u64),
    ToggleDownloading,
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for DownloadsRow {
    type Init = DownloadsRowInit;
    type Input = DownloadsRowMsg;
    type Output = ();

    view! {
        #[root]
        adw::ActionRow {
            add_prefix = model.card.widget(),

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
                    image: init.card_image,
                    title: None,
                    ..CardComponent::small()
                })
                .detach(),
            title: init.title,
            version: init.version,
            edition: init.edition,
            size: init.size,
            downloaded: None,
            downloading: init.start_download,
        };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            DownloadsRowMsg::SetDownloaded(d) => self.downloaded = Some(d),
            DownloadsRowMsg::ToggleDownloading => self.downloading = !self.downloading,
        }
    }
}

#[derive(Debug)]
pub struct DownloadsRowFactory {
    pub card: AsyncController<CardComponent>,
    pub title: Option<&'static str>,
    pub version: Option<&'static str>,
    pub edition: Option<&'static str>,
    pub size: Option<u64>,
    pub downloaded: Option<u64>,
    index: DynamicIndex,
}

#[derive(Debug)]
pub enum DownloadsRowFactoryMsg {
    SetDownloading,
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for DownloadsRowFactory {
    type Init = DownloadsRowInit;
    type Input = DownloadsRowFactoryMsg;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = adw::PreferencesGroup;

    view! {
        #[root]
        adw::ActionRow {
            add_prefix = self.card.widget() {
                set_opacity: 0.5,
            },

            #[watch]
            set_title?: self.title,

            #[watch]
            set_subtitle: &format!("{} ∙ {}", self.edition.unwrap_or("N/A"), self.version.unwrap_or("N/A")),

            add_suffix = &gtk::Label {
                #[watch]
                set_label: &format!("{} / {}", pretty_bytes(self.downloaded.unwrap_or(0)), pretty_bytes(self.size.unwrap_or(0))),
            },

            add_suffix = &gtk::Box {
                set_size_request: (16, 16),
            },

            add_suffix = &gtk::ProgressBar {
                set_align: gtk::Align::Center,
                #[watch]
                set_fraction: self.downloaded.unwrap_or(0) as f64 / self.size.unwrap_or(0) as f64,
            },

            add_suffix = &gtk::Box {
                set_size_request: (16, 16),
            },

            add_suffix = &gtk::Label {
                #[watch]
                set_label: &format!("{:.2}%", self.downloaded.unwrap_or(0) as f64 / self.size.unwrap_or(0) as f64 * 100.0),
            },

            add_suffix = &gtk::Box {
                set_size_request: (16, 16),
            },

            add_suffix = &gtk::Button {
                set_align: gtk::Align::Center,
                #[watch]
                set_icon_name: "download-symbolic",
                connect_clicked => DownloadsRowFactoryMsg::SetDownloading,
            }
        }
    }

    async fn init_model(
        init: Self::Init,
        index: &DynamicIndex,
        sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            card: CardComponent::builder()
                .launch(CardComponent {
                    image: init.card_image,
                    title: None,
                    ..CardComponent::small()
                })
                .detach(),
            title: init.title,
            version: init.version,
            edition: init.edition,
            size: init.size,
            downloaded: None,
            index: index.clone(),
        }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncFactorySender<Self>) {
        match msg {
            DownloadsRowFactoryMsg::SetDownloading => {
                println!("{:?}", self.index);
                todo!("Output index to request download change");
            }
        }
    }
}
