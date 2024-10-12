use adw::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use crate::utils::pretty_bytes;

use super::*;

#[derive(Debug)]
pub struct DownloadsRowInit {
    pub card_image: Option<String>,
    pub title: Option<String>,
    pub version: Option<String>,
    pub edition: Option<String>,
    pub size: Option<u64>,
    pub start: bool,
}

impl DownloadsRowInit {
    #[inline]
    pub fn new(
        card_image: impl ToString,
        title: String,
        version: String,
        edition: String,
        size: u64,
        start: bool,
    ) -> Self {
        Self {
            card_image: Some(card_image.to_string()),
            title: Some(title),
            version: Some(version),
            edition: Some(edition),
            size: Some(size),
            start,
        }
    }
}

#[derive(Debug)]
pub struct DownloadsRow {
    pub card: AsyncController<CardComponent>,
    /// Name of component
    pub title: Option<String>,
    /// Version of component
    pub version: Option<String>,
    /// `Global`, `China`, `TKG`, etc
    pub edition: Option<String>,
    /// Total amount in bytes
    pub size: Option<u64>,
    /// Processed amount in bytes
    pub current_size: Option<u64>,
    /// Indicates if the user has not paused the action
    pub active: bool,
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
            set_title?: &model.title,

            #[watch]
            set_subtitle: &format!("{} ∙ {}", model.edition.clone().unwrap_or(String::from("N/A")), model.version.clone().unwrap_or(String::from("N/A"))),

            add_suffix = &gtk::Label {
                #[watch]
                set_label: {
                    let curr = pretty_bytes(model.current_size.unwrap_or(0));
                    let total = pretty_bytes(model.size.unwrap_or(0));

                    &format!("{} {} / {} {}", curr.0, curr.1, total.0, total.1)
                }
            },

            add_suffix = &gtk::Box {
                set_size_request: (16, 16),
            },

            add_suffix = &gtk::ProgressBar {
                set_align: gtk::Align::Center,
                #[watch]
                set_fraction: model.current_size.unwrap_or(0) as f64 / model.size.unwrap_or(0) as f64,
            },

            add_suffix = &gtk::Box {
                set_size_request: (16, 16),
            },

            add_suffix = &gtk::Label {
                #[watch]
                set_label: &format!("{:.2}%", model.current_size.unwrap_or(0) as f64 / model.size.unwrap_or(0) as f64 * 100.0),
            },

            add_suffix = &gtk::Box {
                set_size_request: (16, 16),
            },

            add_suffix = &gtk::Button {
                set_align: gtk::Align::Center,
                #[watch]
                set_icon_name: if model.active {"media-playback-pause-symbolic"} else {"media-playback-start-symbolic"},
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
                .launch(CardComponent::small())
                .detach(),
            title: init.title,
            version: init.version,
            edition: init.edition,
            size: init.size,
            current_size: None,
            active: init.start,
        };
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            DownloadsRowMsg::SetDownloaded(d) => self.current_size = Some(d),
            DownloadsRowMsg::ToggleDownloading => self.active = !self.active,
        }
    }
}

#[derive(Debug)]
pub struct DownloadsRowFactory {
    pub card: AsyncController<CardComponent>,
    pub title: Option<String>,
    pub version: Option<String>,
    pub edition: Option<String>,
    pub size: Option<u64>,
    pub current_size: Option<u64>,
    index: DynamicIndex,
}

#[derive(Debug)]
pub enum DownloadsRowFactoryMsg {
    SetActive,
    UpdateCurrentSize(u64),
}

#[derive(Debug)]
pub enum DownloadsRowFactoryOutput {
    Queue(DynamicIndex),
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for DownloadsRowFactory {
    type Init = DownloadsRowInit;
    type Input = DownloadsRowFactoryMsg;
    type Output = DownloadsRowFactoryOutput;
    type CommandOutput = ();
    type ParentWidget = adw::PreferencesGroup;

    view! {
        #[root]
        adw::ActionRow {
            add_prefix = self.card.widget() {
                set_opacity: 0.5,
            },

            #[watch]
            set_title?: &self.title,

            #[watch]
            set_subtitle: &format!("{} ∙ {}", self.edition.clone().unwrap_or(String::from("N/A")), self.version.clone().unwrap_or(String::from("N/A"))),

            add_suffix = &gtk::Label {
                #[watch]
                set_label: {
                    let curr = pretty_bytes(self.current_size.unwrap_or(0));
                    let total = pretty_bytes(self.size.unwrap_or(0));

                    &format!("{} {} / {} {}", curr.0, curr.1, total.0, total.1)
                }
            },

            add_suffix = &gtk::Box {
                set_size_request: (16, 16),
            },

            add_suffix = &gtk::ProgressBar {
                set_align: gtk::Align::Center,
                #[watch]
                set_fraction: self.current_size.unwrap_or(0) as f64 / self.size.unwrap_or(0) as f64,
            },

            add_suffix = &gtk::Box {
                set_size_request: (16, 16),
            },

            add_suffix = &gtk::Label {
                #[watch]
                set_label: &format!("{:.2}%", self.current_size.unwrap_or(0) as f64 / self.size.unwrap_or(0) as f64 * 100.0),
            },

            add_suffix = &gtk::Box {
                set_size_request: (16, 16),
            },

            add_suffix = &gtk::Button {
                set_align: gtk::Align::Center,
                #[watch]
                set_icon_name: "download-symbolic",
                connect_clicked => DownloadsRowFactoryMsg::SetActive,
            }
        }
    }

    async fn init_model(
        init: Self::Init,
        index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self {
            card: CardComponent::builder()
                .launch(CardComponent::small())
                .detach(),
            title: init.title,
            version: init.version,
            edition: init.edition,
            size: init.size,
            current_size: None,
            index: index.clone(),
        }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncFactorySender<Self>) {
        match msg {
            DownloadsRowFactoryMsg::SetActive => {
                sender
                    .output(DownloadsRowFactoryOutput::Queue(self.index.clone()))
                    .unwrap();
            }
            DownloadsRowFactoryMsg::UpdateCurrentSize(size) => {
                self.current_size = Some(size);
            }
        }
    }
}
