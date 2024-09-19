use adw::prelude::*;
use gtk::prelude::*;

use relm4::prelude::*;

use hardware_requirements::{requirements::HardwareRequirements, GameHardwareRequirements};
use unic_langid::LanguageIdentifier;

use crate::{
    games::manifest::info::*,
    utils::{pretty_bytes, pretty_frequency},
};

const DIM_CLASS: &[&str] = &["dim-label"];

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
                    set_label: &model.cpu.as_ref().unwrap().model.translate(&lang),
                        set_css_classes: DIM_CLASS,
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Core Count",
                        add_suffix = &gtk::Label {
                            set_label: &model.cpu.as_ref().unwrap().cores.unwrap().to_string(),
                            set_css_classes: DIM_CLASS,
                        }
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Frequency",
                        add_suffix = &gtk::Label {
                            set_label: &pretty_frequency(model.cpu.as_ref().unwrap().frequency.unwrap()),
                            set_css_classes: DIM_CLASS,
                        }
                    }
                },

                adw::ExpanderRow {
                    set_title: "Graphics",
                    add_suffix = &gtk::Label {
                        set_label: &model.gpu.as_ref().unwrap().model.translate(&lang),
                        set_css_classes: DIM_CLASS,
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Video Memory",
                        add_suffix = &gtk::Label {
                            set_label: &pretty_bytes(model.gpu.as_ref().unwrap().vram.unwrap()),
                            set_css_classes: DIM_CLASS,
                        }
                    }
                },

                adw::ExpanderRow {
                    set_title: "Memory",
                    add_suffix = &gtk::Label {
                        set_label: &model.cpu.as_ref().unwrap().model.translate(&lang),
                        set_css_classes: DIM_CLASS,
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Capacity",
                        add_suffix = &gtk::Label {
                            set_label: &pretty_bytes(model.ram.as_ref().unwrap().size),
                            set_css_classes: DIM_CLASS,
                        }
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Frequency",
                        add_suffix = &gtk::Label {
                            set_label: &pretty_frequency(model.ram.as_ref().unwrap().frequency.unwrap()),
                            set_css_classes: DIM_CLASS,
                        }
                    }
                },

                adw::ExpanderRow {
                    set_title: "Disk",
                    add_suffix = &gtk::Label {
                        set_label: &pretty_bytes(model.disk.as_ref().unwrap().size),
                        set_css_classes: DIM_CLASS,
                    },
                    add_row = &adw::ActionRow {
                        set_title: "Type",
                        add_suffix = &gtk::Label {
                            set_label: &format!("{}", model.disk.as_ref().unwrap().disk_type.clone().unwrap()).to_uppercase(),
                            set_css_classes: DIM_CLASS,
                        }
                    }
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
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
            view_stack -> adw::ViewStack {
                add = &gtk::Box {
                    model.minimal.widget(),
                } -> {
                    set_name: Some("Minimum"),
                    set_title: Some("Minimum"),
                    set_icon_name: Some("speedometer4-symbolic"),
                },
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let view_stack = adw::ViewStack::new();
        let lang = LanguageIdentifier::default();

        let mut model = Self {
            minimal: HardwareRequirements::builder()
                .launch((init.minimal, lang.clone()))
                .detach(),
            optimal: None,
        };

        // Insert recommended only if present
        if let Some(optimal) = init.optimal {
            model.optimal = Some(
                HardwareRequirements::builder()
                    .launch((optimal, lang))
                    .detach(),
            );
        }
        if let Some(req) = &model.optimal {
            view_stack
                .add_titled(req.widget(), None, "Recommended")
                .set_icon_name(Some("speedometer2-symbolic"));
        }

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }
}
