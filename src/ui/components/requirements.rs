use adw::prelude::*;
use gtk::prelude::*;

use relm4::prelude::*;

use hardware_requirements::{requirements::HardwareRequirements, GameHardwareRequirements};
use unic_langid::LanguageIdentifier;

use crate::{
    games::manifest::info::*,
    utils::{pretty_bytes, pretty_frequency},
};

const DIM_CLASS: &str = "dim-label";

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for HardwareRequirements {
    type Init = (HardwareRequirements, LanguageIdentifier);
    type Input = ();
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
                        set_label: model.cpu.as_ref().map(|a| a.model.translate(&lang)).unwrap_or("N/A"),
                        add_css_class: DIM_CLASS,
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Core Count",
                        add_suffix = &gtk::Label {
                            set_label: &model.cpu.as_ref().and_then(|a| a.cores).map(|b| b.to_string()).unwrap_or("N/A".to_string()),
                            add_css_class: DIM_CLASS,
                        }
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Frequency",
                        add_suffix = &gtk::Label {
                            set_label: &model.cpu.as_ref().and_then(|a| a.frequency).map(|b| pretty_frequency(b, false)).unwrap_or("N/A".to_string()),
                            add_css_class: DIM_CLASS,
                        }
                    }
                },

                adw::ExpanderRow {
                    set_title: "Graphics",
                    add_suffix = &gtk::Label {
                        set_label: model.gpu.as_ref().map(|a| a.model.translate(&lang)).unwrap_or("N/A"),
                        add_css_class: DIM_CLASS,
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Video Memory",
                        add_suffix = &gtk::Label {
                            set_label: &model.gpu.as_ref().and_then(|a| a.vram).map(|b| pretty_bytes(b)).unwrap_or("N/A".to_string()),
                            add_css_class: DIM_CLASS,
                        }
                    }
                },

                adw::ExpanderRow {
                    set_title: "Memory",
                    add_suffix = &gtk::Label {
                        set_label: &model.ram.as_ref().map(|a| pretty_bytes(a.size)).unwrap_or("N/A".to_string()),
                        add_css_class: DIM_CLASS,
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Frequency",
                        add_suffix = &gtk::Label {
                            set_label: &model.ram.as_ref().and_then(|a| a.frequency).map(|b| pretty_frequency(b, true)).unwrap_or("N/A".to_string()),
                            add_css_class: DIM_CLASS,
                        }
                    }
                },

                adw::ExpanderRow {
                    set_title: "Disk",
                    add_suffix = &gtk::Label {
                        set_label: &model.disk.as_ref().map(|a| pretty_bytes(a.size)).unwrap_or("N/A".to_string()),
                        add_css_class: DIM_CLASS,
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Type",
                        add_suffix = &gtk::Label {
                            set_label: &model.disk.as_ref().and_then(|a| a.disk_type.as_ref()).map(|b| b.to_string().to_uppercase()).unwrap_or("N/A".to_string()),
                            add_css_class: DIM_CLASS,
                        }
                    }
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = init.0;
        let lang = init.1;

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }
}

#[derive(Debug)]
pub struct RequirementsComponent {
    pub minimal: AsyncController<HardwareRequirements>,
    pub optimal: Option<AsyncController<HardwareRequirements>>,
}

#[relm4::component(async, pub)]
impl SimpleAsyncComponent for RequirementsComponent {
    type Init = GameHardwareRequirements;
    type Input = ();
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 16,

            adw::ViewSwitcher {
                set_policy: adw::ViewSwitcherPolicy::Wide,
                set_stack = Some(&view_stack),
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
        let view_stack = adw::ViewStack::new();
        let lang = LanguageIdentifier::default();

        let mut model = Self {
            minimal: HardwareRequirements::builder()
                .launch((init.minimal, lang.clone()))
                .detach(),
            optimal: None,
        };

        // Insert minimum
        view_stack.add_titled_with_icon(
            model.minimal.widget(),
            None,
            "Minimum",
            "speedometer4-symbolic",
        );

        // Insert recommended only if present
        if let Some(optimal) = init.optimal {
            model.optimal = Some(
                HardwareRequirements::builder()
                    .launch((optimal, lang))
                    .detach(),
            );
        }
        if let Some(req) = &model.optimal {
            view_stack.add_titled_with_icon(
                req.widget(),
                None,
                "Recommended",
                "speedometer2-symbolic",
            );
        }

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }
}
