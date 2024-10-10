use gtk::prelude::*;
use adw::prelude::*;
use relm4::prelude::*;

use hardware_requirements::{requirements::HardwareRequirements, GameHardwareRequirements};
use unic_langid::LanguageIdentifier;

use crate::{
    games::manifest::info::*,
    utils::{pretty_bytes, pretty_frequency},
};

const DIM_CLASS: &str = "dim-label";

#[derive(Debug)]
pub struct HardwareRequirementsComponent {
    req: HardwareRequirements,
    lang: LanguageIdentifier,
}

#[derive(Debug)]
pub enum HardwareRequirementsComponentMsg {
    Update(HardwareRequirementsComponent),
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for HardwareRequirementsComponent {
    type Init = HardwareRequirementsComponent;
    type Input = HardwareRequirementsComponentMsg;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_hexpand: true,
            set_vexpand: true,

            adw::PreferencesGroup {
                set_hexpand: true,
                adw::ExpanderRow {
                    set_title: "Processor",
                    add_suffix = &gtk::Label {
                        #[watch]
                        set_label: model.req.cpu.as_ref().map(|a| a.model.translate(&model.lang)).unwrap_or("N/A"),
                        add_css_class: DIM_CLASS,
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Core Count",
                        add_suffix = &gtk::Label {
                            #[watch]
                            set_label: &model.req.cpu.as_ref().and_then(|a| a.cores).map(|b| b.to_string()).unwrap_or("N/A".to_string()),
                            add_css_class: DIM_CLASS,
                        }
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Frequency",
                        add_suffix = &gtk::Label {
                            #[watch]
                            set_label: &model.req.cpu.as_ref().and_then(|a| a.frequency).map(|b| pretty_frequency(b, false)).unwrap_or("N/A".to_string()),
                            add_css_class: DIM_CLASS,
                        }
                    }
                },

                adw::ExpanderRow {
                    set_title: "Graphics",
                    add_suffix = &gtk::Label {
                        #[watch]
                        set_label: model.req.gpu.as_ref().map(|a| a.model.translate(&model.lang)).unwrap_or("N/A"),
                        add_css_class: DIM_CLASS,
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Video Memory",
                        add_suffix = &gtk::Label {
                            #[watch]
                            set_label: &model.req.gpu.as_ref().and_then(|a| a.vram).map(|b| pretty_bytes(b)).unwrap_or("N/A".to_string()),
                            add_css_class: DIM_CLASS,
                        }
                    }
                },

                adw::ExpanderRow {
                    set_title: "Memory",
                    add_suffix = &gtk::Label {
                        #[watch]
                        set_label: &model.req.ram.as_ref().map(|a| pretty_bytes(a.size)).unwrap_or("N/A".to_string()),
                        add_css_class: DIM_CLASS,
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Frequency",
                        add_suffix = &gtk::Label {
                            #[watch]
                            set_label: &model.req.ram.as_ref().and_then(|a| a.frequency).map(|b| pretty_frequency(b, true)).unwrap_or("N/A".to_string()),
                            add_css_class: DIM_CLASS,
                        }
                    }
                },

                adw::ExpanderRow {
                    set_title: "Disk",
                    add_suffix = &gtk::Label {
                        #[watch]
                        set_label: &model.req.disk.as_ref().map(|a| pretty_bytes(a.size)).unwrap_or("N/A".to_string()),
                        add_css_class: DIM_CLASS,
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Type",
                        add_suffix = &gtk::Label {
                            #[watch]
                            set_label: &model.req.disk.as_ref().and_then(|a| a.disk_type.as_ref()).map(|b| b.to_string().to_uppercase()).unwrap_or("N/A".to_string()),
                            add_css_class: DIM_CLASS,
                        }
                    }
                }
            }
        }
    }

    async fn init(
        model: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            HardwareRequirementsComponentMsg::Update(req) => {
                self.req = req.req;
                self.lang = req.lang;
            }
        }
    }
}

#[derive(Debug)]
pub struct RequirementsComponent {
    pub minimal: AsyncController<HardwareRequirementsComponent>,
    pub optimal: Option<AsyncController<HardwareRequirementsComponent>>,
    stack: adw::ViewStack,
}

#[derive(Debug)]
pub enum RequirementsComponentMsg {
    Update((GameHardwareRequirements, LanguageIdentifier)),
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for RequirementsComponent {
    type Init = (GameHardwareRequirements, LanguageIdentifier);
    type Input = RequirementsComponentMsg;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 16,

            adw::ViewSwitcher {
                set_policy: adw::ViewSwitcherPolicy::Wide,
                set_stack = Some(view_stack),
            },

            #[local_ref]
            view_stack -> adw::ViewStack,
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let mut model = Self {
            minimal: HardwareRequirementsComponent::builder()
                .launch(HardwareRequirementsComponent {
                    req: init.0.minimal,
                    lang: init.1.clone(),
                })
                .detach(),
            optimal: None,
            stack: adw::ViewStack::new(),
        };

        let view_stack = &model.stack;

        // Insert minimum
        view_stack.add_titled_with_icon(
            model.minimal.widget(),
            None,
            "Minimum",
            "speedometer4-symbolic",
        );

        // Insert recommended only if present
        if let Some(optimal) = init.0.optimal {
            model.optimal = Some(
                HardwareRequirementsComponent::builder()
                    .launch(HardwareRequirementsComponent {
                        req: optimal,
                        lang: init.1,
                    })
                    .detach(),
            );
        }
        if let Some(req) = &model.optimal {
            view_stack.add_titled_with_icon(
                req.widget(),
                Some("recommended"),
                "Recommended",
                "speedometer2-symbolic",
            );
        }

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            RequirementsComponentMsg::Update((requirements, lang)) => {
                self.minimal
                    .sender()
                    .send(HardwareRequirementsComponentMsg::Update(
                        HardwareRequirementsComponent {
                            req: requirements.minimal,
                            lang: lang.clone(),
                        },
                    ))
                    .unwrap();

                match requirements.optimal {
                    Some(opt) => match &self.optimal {
                        Some(optimal) => {
                            optimal
                                .sender()
                                .send(HardwareRequirementsComponentMsg::Update(
                                    HardwareRequirementsComponent {
                                        req: opt,
                                        lang: lang.clone(),
                                    },
                                ))
                                .unwrap();
                        }
                        None => {
                            self.optimal = Some(
                                HardwareRequirementsComponent::builder()
                                    .launch(HardwareRequirementsComponent { req: opt, lang })
                                    .detach(),
                            );
                            if let Some(optimal) = &self.optimal {
                                // Remove optimal if presend
                                if let Some(child) = self.stack.child_by_name("recommended") {
                                    self.stack.remove(&child);
                                }

                                // Add new optimal
                                self.stack.add_titled_with_icon(
                                    optimal.widget(),
                                    Some("recommended"),
                                    "Recommended",
                                    "speedometer2-symbolic",
                                );
                            }
                        }
                    },
                    None => self.optimal = None,
                }
            }
        }
    }
}
