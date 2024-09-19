use adw::prelude::*;
use gtk::prelude::*;

use relm4::prelude::*;

use hardware_requirements::GameHardwareRequirements;
use unic_langid::LanguageIdentifier;

use crate::{
    games::manifest::info::*,
    utils::{pretty_bytes, pretty_frequency},
};

const DIM_CLASS: &[&str] = &["dim-label"];

#[derive(Debug)]
pub struct RequirementsComponent {
    pub req: GameHardwareRequirements,
    pub lang: LanguageIdentifier,
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
                    set_hexpand: true,
                    set_vexpand: true,

                    adw::PreferencesGroup {
                        set_hexpand: true,
                        adw::ExpanderRow {
                            set_title: "Processor",
                            add_suffix = &gtk::Label {
                            set_label: &model.req.minimal.cpu.as_ref().unwrap().model.translate(&model.lang),
                                set_css_classes: DIM_CLASS,
                            },
                            add_row = &adw::ActionRow {
                                set_title: "Core Count",
                                add_suffix = &gtk::Label {
                                    set_label: &model.req.minimal.cpu.as_ref().unwrap().cores.unwrap().to_string(),
                                    set_css_classes: DIM_CLASS,
                                }
                            },
                            add_row = &adw::ActionRow {
                                set_title: "Frequency",
                                add_suffix = &gtk::Label {
                                    set_label: &pretty_frequency(model.req.minimal.cpu.as_ref().unwrap().frequency.unwrap()),
                                    set_css_classes: DIM_CLASS,
                                }
                            }
                        },

                        adw::ExpanderRow {
                            set_title: "Graphics",
                            add_suffix = &gtk::Label {
                                set_label: &model.req.minimal.gpu.as_ref().unwrap().model.translate(&model.lang),
                                set_css_classes: DIM_CLASS,
                            },
                            add_row = &adw::ActionRow {
                                set_title: "Video Memory",
                                add_suffix = &gtk::Label {
                                    set_label: &pretty_bytes(model.req.minimal.gpu.as_ref().unwrap().vram.unwrap()),
                                    set_css_classes: DIM_CLASS,
                                }
                            }
                        },

                        adw::ExpanderRow {
                            set_title: "Memory",
                            add_suffix = &gtk::Label {
                                set_label: &model.req.minimal.cpu.as_ref().unwrap().model.translate(&model.lang),
                                set_css_classes: DIM_CLASS,
                            },
                            add_row = &adw::ActionRow {
                                set_title: "Capacity",
                                add_suffix = &gtk::Label {
                                    set_label: &pretty_bytes(model.req.minimal.ram.as_ref().unwrap().size),
                                    set_css_classes: DIM_CLASS,
                                }
                            },
                            add_row = &adw::ActionRow {
                                set_title: "Frequency",
                                add_suffix = &gtk::Label {
                                    set_label: &pretty_frequency(model.req.minimal.ram.as_ref().unwrap().frequency.unwrap()),
                                    set_css_classes: DIM_CLASS,
                                }
                            }
                        },

                        adw::ExpanderRow {
                            set_title: "Disk",
                            add_suffix = &gtk::Label {
                                set_label: &pretty_bytes(model.req.minimal.disk.as_ref().unwrap().size),
                                set_css_classes: DIM_CLASS,
                            },
                            add_row = &adw::ActionRow {
                                set_title: "Type",
                                add_suffix = &gtk::Label {
                                    set_label: &format!("{}", model.req.minimal.disk.as_ref().unwrap().disk_type.clone().unwrap()).to_uppercase(),
                                    set_css_classes: DIM_CLASS,
                                }
                            }
                        }
                    }
                } -> {
                    set_name: Some("Minimum"),
                    set_title: Some("Minimum"),
                    set_icon_name: Some("speedometer4-symbolic"),
                },

                add = &gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,

                    adw::PreferencesGroup {
                        set_hexpand: true,
                        adw::ExpanderRow {
                            set_title: "Processor",
                            add_suffix = &gtk::Label {
                            set_label: &opt.clone().unwrap().cpu.as_ref().unwrap().model.translate(&model.lang),
                                set_css_classes: DIM_CLASS,
                            },
                            add_row = &adw::ActionRow {
                                set_title: "Core Count",
                                add_suffix = &gtk::Label {
                                    set_label: &opt.clone().unwrap().cpu.as_ref().unwrap().cores.unwrap().to_string(),
                                    set_css_classes: DIM_CLASS,
                                }
                            },
                            add_row = &adw::ActionRow {
                                set_title: "Frequency",
                                add_suffix = &gtk::Label {
                                    set_label: &pretty_frequency(opt.clone().unwrap().cpu.as_ref().unwrap().frequency.unwrap()),
                                    set_css_classes: DIM_CLASS,
                                }
                            }
                        },

                        adw::ExpanderRow {
                            set_title: "Graphics",
                            add_suffix = &gtk::Label {
                                set_label: &opt.clone().unwrap().gpu.as_ref().unwrap().model.translate(&model.lang),
                                set_css_classes: DIM_CLASS,
                            },
                            add_row = &adw::ActionRow {
                                set_title: "Video Memory",
                                add_suffix = &gtk::Label {
                                    set_label: &pretty_bytes(opt.clone().unwrap().gpu.as_ref().unwrap().vram.unwrap()),
                                    set_css_classes: DIM_CLASS,
                                }
                            }
                        },

                        adw::ExpanderRow {
                            set_title: "Memory",
                            add_suffix = &gtk::Label {
                                set_label: &opt.clone().unwrap().cpu.as_ref().unwrap().model.translate(&model.lang),
                                set_css_classes: DIM_CLASS,
                            },
                            add_row = &adw::ActionRow {
                                set_title: "Capacity",
                                add_suffix = &gtk::Label {
                                    set_label: &pretty_bytes(opt.clone().unwrap().ram.as_ref().unwrap().size),
                                    set_css_classes: DIM_CLASS,
                                }
                            },
                            add_row = &adw::ActionRow {
                                set_title: "Frequency",
                                add_suffix = &gtk::Label {
                                    set_label: &pretty_frequency(opt.clone().unwrap().ram.as_ref().unwrap().frequency.unwrap()),
                                    set_css_classes: DIM_CLASS,
                                }
                            }
                        },

                        adw::ExpanderRow {
                            set_title: "Disk",
                            add_suffix = &gtk::Label {
                                set_label: &pretty_bytes(opt.clone().unwrap().disk.as_ref().unwrap().size),
                                set_css_classes: DIM_CLASS,
                            },
                            add_row = &adw::ActionRow {
                                set_title: "Type",
                                add_suffix = &gtk::Label {
                                    set_label: &format!("{}", opt.clone().unwrap().disk.as_ref().unwrap().disk_type.clone().unwrap()).to_uppercase(),
                                    set_css_classes: DIM_CLASS,
                                }
                            }
                        }
                    }
                } -> {
                    set_name: Some("Recommended"),
                    set_title: Some("Recommended"),
                    set_icon_name: Some("speedometer2-symbolic"),
                }
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let view_stack = adw::ViewStack::new();
        let model = Self {
            req: init,
            lang: LanguageIdentifier::default(),
        };
        let opt = &model.req.optimal;
        let widgets = view_output!();
        AsyncComponentParts { model, widgets }
    }
}
