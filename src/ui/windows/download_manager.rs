use std::time::Instant;

use adw::prelude::*;
use relm4::prelude::*;

use unic_langid::LanguageIdentifier;

use crate::prelude::*;

static mut WINDOW: Option<adw::Window> = None;

pub struct PipelineActionHandlers {
    pub before: Box<dyn Fn(PipelineActionProgressReport) -> bool + Send + Sync>,
    pub perform: Box<dyn Fn(PipelineActionProgressReport) + Send + Sync>,
    pub after: Box<dyn Fn(PipelineActionProgressReport) -> bool + Send + Sync>
}

impl std::fmt::Debug for PipelineActionHandlers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineActionHandlers").finish()
    }
}

#[derive(Debug, Clone)]
pub struct PipelineActionProgressReport {
    pub title: Option<LocalizableString>,
    pub description: Option<LocalizableString>,
    pub progress: LocalizableString,
    pub current: u64,
    pub total: u64
}

#[derive(Debug)]
pub struct DownloadManagerWindow {
    graph: AsyncController<Graph>,

    progress_bar: gtk::ProgressBar,

    updates_lang: Option<LanguageIdentifier>,

    diff_title: Option<String>,
    diff_description: Option<String>,

    action_title: Option<String>,
    action_description: Option<String>,

    last_current: u64,
    last_update: Instant
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum DownloadManagerWindowMsg {
    Show,
    Hide,

    PrepareAction {
        diff_title: LocalizableString,
        diff_description: Option<LocalizableString>,

        action_title: LocalizableString,
        action_description: Option<LocalizableString>,

        handlers_listener: flume::Sender<PipelineActionHandlers>
    },

    UpdateProgress(PipelineActionProgressReport)
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for DownloadManagerWindow {
    type Init = ();
    type Input = DownloadManagerWindowMsg;
    type Output = ();

    view! {
        window = adw::Window {
            set_size_request: (700, 560),
            set_title: Some("Download manager"),

            set_hide_on_close: true,
            set_modal: true,

            add_css_class?: APP_DEBUG.then_some("devel"),

            adw::Clamp {
                set_maximum_size: 600,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Label {
                        set_align: gtk::Align::Start,

                        add_css_class: "title-1",

                        #[watch]
                        set_label?: model.diff_title.as_deref(),

                        #[watch]
                        set_tooltip?: model.diff_description.as_deref()
                    },

                    gtk::Label {
                        set_align: gtk::Align::Start,

                        #[watch]
                        set_label?: model.action_title.as_deref(),

                        #[watch]
                        set_tooltip?: model.action_description.as_deref()
                    },

                    model.graph.widget(),

                    #[local_ref]
                    progress_bar -> gtk::ProgressBar {
                        set_show_text: true
                    }
                }
            }

            // adw::PreferencesPage {
            //     adw::PreferencesGroup {
            //         model.graph.widget(),
            //     },

            //     adw::PreferencesGroup {
            //         gtk::Box {
            //             set_orientation: gtk::Orientation::Horizontal,
            //             set_spacing: 16,

            //             // adw::PreferencesGroup {
            //             //     adw::ActionRow {
            //             //         set_title: "Current speed",
            //             //         #[watch]
            //             //         set_subtitle: &format!("{}/s", pretty_bytes(model.speed).0),
            //             //     }
            //             // },

            //             // adw::PreferencesGroup {
            //             //     adw::ActionRow {
            //             //         set_title: "Average speed",
            //             //         #[watch]
            //             //         set_subtitle: &format!("{}/s", pretty_bytes(model.avg_speed).0),
            //             //     }
            //             // },

            //             // adw::PreferencesGroup {
            //             //     adw::ActionRow {
            //             //         set_title: "Time elapsed",
            //             //         #[watch]
            //             //         set_subtitle: &pretty_seconds(model.elapsed),
            //             //     }
            //             // },

            //             // adw::PreferencesGroup {
            //             //     adw::ActionRow {
            //             //         set_title: "Current ETA",
            //             //         set_subtitle: "amogus",
            //             //     }
            //             // },

            //             // adw::PreferencesGroup {
            //             //     adw::ActionRow {
            //             //         #[watch]
            //             //         set_title: match model.state {
            //             //             DownloadsAppState::None => "",
            //             //             DownloadsAppState::Downloading => "Total download",
            //             //             DownloadsAppState::Extracting => "Total extracted",
            //             //             DownloadsAppState::StreamUnpacking => "Total unpacked",
            //             //             DownloadsAppState::Verifying => "Total verified",
            //             //         },
    
            //             //         #[watch]
            //             //         set_subtitle: &pretty_bytes(model.total).0.to_string(),
            //             //     }
            //             // }
            //         }
            //     }
            // }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, _sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            graph: Graph::builder()
                .launch(GraphInit {
                    width: 600,
                    height: 300,
                    window_size: 200,
                    color: (1.0, 0.0, 0.0)
                })
                .detach(),

            progress_bar: gtk::ProgressBar::new(),

            updates_lang: None,

            diff_title: None,
            diff_description: None,

            action_title: None,
            action_description: None,

            last_current: 0,
            last_update: Instant::now()
        };

        let progress_bar = &model.progress_bar;

        let widgets = view_output!();

        unsafe {
            WINDOW = Some(widgets.window.clone());
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            DownloadManagerWindowMsg::Show => unsafe {
                if let Some(window) = WINDOW.as_ref() {
                    if let Some(main_window) = MAIN_WINDOW.as_ref() {
                        let main_window = main_window.upcast_ref::<gtk::Window>();

                        window.set_transient_for(Some(main_window));
                    }

                    window.present();
                }
            }

            DownloadManagerWindowMsg::Hide => unsafe {
                if let Some(window) = WINDOW.as_ref() {
                    window.close();
                }
            }

            DownloadManagerWindowMsg::PrepareAction {
                diff_title,
                diff_description,
                action_title,
                action_description,
                handlers_listener
            } => {
                let config = config::get();

                let lang = config.general.language.parse::<LanguageIdentifier>();

                if let Ok(lang) = &lang {
                    self.updates_lang = Some(lang.clone());
                }

                self.graph.emit(GraphMsg::Clear);

                self.diff_title = match &lang {
                    Ok(lang) => Some(diff_title.translate(lang).to_string()),
                    Err(_) => Some(diff_title.default_translation().to_string())
                };

                self.diff_description = match diff_description {
                    Some(diff_description) => match &lang {
                        Ok(lang) => Some(diff_description.translate(lang).to_string()),
                        Err(_) => Some(diff_description.default_translation().to_string())
                    },

                    None => None
                };

                self.action_title = match &lang {
                    Ok(lang) => Some(action_title.translate(lang).to_string()),
                    Err(_) => Some(action_title.default_translation().to_string())
                };

                self.action_description = match action_description {
                    Some(action_description) => match &lang {
                        Ok(lang) => Some(action_description.translate(lang).to_string()),
                        Err(_) => Some(action_description.default_translation().to_string())
                    },

                    None => None
                };

                self.last_current = 0;

                // TODO: handle errors
                // TODO: proper before/after hooks
                let result = handlers_listener.send(PipelineActionHandlers {
                    before: {
                        let sender = sender.input_sender().clone();

                        Box::new(move |progress| {
                            let _ = sender.send(DownloadManagerWindowMsg::UpdateProgress(progress));

                            true
                        })
                    },

                    perform: {
                        let sender = sender.input_sender().clone();

                        Box::new(move |progress| {
                            let _ = sender.send(DownloadManagerWindowMsg::UpdateProgress(progress));
                        })
                    },

                    after: {
                        let sender = sender.input_sender().clone();

                        Box::new(move |progress| {
                            let _ = sender.send(DownloadManagerWindowMsg::UpdateProgress(progress));
    
                            true
                        })
                    }
                });

                if let Err(err) = result {
                    tracing::error!(?err, "Failed to send pipeline action handlers");
                }
            }

            DownloadManagerWindowMsg::UpdateProgress(progress) => {
                let progress_label = match &self.updates_lang {
                    Some(lang) => progress.progress.translate(lang),
                    None => progress.progress.default_translation()
                };

                self.progress_bar.set_text(Some(progress_label));
                self.progress_bar.set_fraction(progress.current as f64 / progress.total as f64);

                if self.last_update.elapsed().as_millis() > 1000 {
                    self.graph.emit(GraphMsg::AddPoint(progress.current - self.last_current));

                    if let Some(title) = progress.title {
                        let title = match &self.updates_lang {
                            Some(lang) => title.translate(lang),
                            None => title.default_translation()
                        };

                        // TODO
                    }

                    if let Some(description) = progress.description {
                        let description = match &self.updates_lang {
                            Some(lang) => description.translate(lang),
                            None => description.default_translation()
                        };

                        // TODO
                    }

                    self.last_current = progress.current;
                    self.last_update = Instant::now();
                }
            }
        }
    }
}
