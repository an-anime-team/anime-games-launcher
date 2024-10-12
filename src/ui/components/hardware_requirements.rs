use adw::prelude::*;
use relm4::prelude::*;

use unic_langid::LanguageIdentifier;

use crate::prelude::*;

use crate::games::manifest::info::hardware_requirements::GameHardwareRequirements;
use crate::games::manifest::info::hardware_requirements::disk_type::DiskType;
use crate::games::manifest::info::hardware_requirements::requirements::HardwareRequirements;

#[derive(Debug)]
pub struct HardwareRequirementsComponent {
    pub minimal: AsyncController<HardwareRequirementsSection>,
    pub optimal: AsyncController<HardwareRequirementsSection>,

    minimal_page: adw::ViewStackPage,
    optimal_page: adw::ViewStackPage
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HardwareRequirementsComponentMsg {
    SetRequirements(GameHardwareRequirements),
    Clear
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for HardwareRequirementsComponent {
    type Init = ();
    type Input = HardwareRequirementsComponentMsg;
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

    async fn init(_init: Self::Init, root: Self::Root, _sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let minimal = HardwareRequirementsSection::builder()
            .launch(())
            .detach();

        let optimal = HardwareRequirementsSection::builder()
            .launch(())
            .detach();

        let view_stack = &adw::ViewStack::new();

        let minimal_page = view_stack.add_titled_with_icon(minimal.widget(), None, "Minimum", "speedometer4-symbolic");
        let optimal_page = view_stack.add_titled_with_icon(optimal.widget(), None, "Recommended", "speedometer2-symbolic");

        let model = Self {
            minimal,
            optimal,
            minimal_page,
            optimal_page
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            HardwareRequirementsComponentMsg::SetRequirements(requirements) => {
                let language = config::get().general.language.parse::<LanguageIdentifier>().ok();

                // TODO: proper widgets hiding

                sender.input(HardwareRequirementsComponentMsg::Clear);

                self.minimal_page.set_visible(true);

                self.minimal.emit(HardwareRequirementsSectionMsg::SetRequirements {
                    requirements: requirements.minimal,
                    language: language.clone()
                });

                if let Some(requirements) = requirements.optimal {
                    self.optimal_page.set_visible(true);

                    self.optimal.emit(HardwareRequirementsSectionMsg::SetRequirements {
                        requirements,
                        language
                    });
                }
            }

            HardwareRequirementsComponentMsg::Clear => {
                self.minimal.emit(HardwareRequirementsSectionMsg::Clear);
                self.optimal.emit(HardwareRequirementsSectionMsg::Clear);

                self.minimal_page.set_visible(false);
                self.optimal_page.set_visible(false);
            }
        }
    }
}

#[derive(Debug)]
pub struct HardwareRequirementsSection {
    pub requirements: HardwareRequirements,
    pub language: Option<LanguageIdentifier>
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HardwareRequirementsSectionMsg {
    SetRequirements {
        requirements: HardwareRequirements,
        language: Option<LanguageIdentifier>
    },

    Clear
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for HardwareRequirementsSection {
    type Init = ();
    type Input = HardwareRequirementsSectionMsg;
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

                    #[watch]
                    set_visible: model.requirements.cpu.is_some(),

                    add_suffix = &gtk::Label {
                        add_css_class: "dim-label",

                        #[watch]
                        set_label?: model.requirements.cpu.as_ref()
                            .map(|cpu| {
                                match &model.language {
                                    Some(lang) => cpu.model.translate(lang),
                                    None => cpu.model.default_translation()
                                }
                            })
                    },

                    add_row = &adw::ActionRow {
                        set_title: "Cores",

                        #[watch]
                        set_visible: model.requirements.cpu.as_ref()
                            .map(|cpu| cpu.cores.is_some())
                            .unwrap_or_default(),

                        add_suffix = &gtk::Label {
                            add_css_class: "dim-label",

                            #[watch]
                            set_label?: model.requirements.cpu.as_ref()
                                .and_then(|cpu| cpu.cores.map(|cores| cores.to_string()))
                                .as_deref()
                        }
                    },

                    add_row = &adw::ActionRow {
                        set_title: "Frequency",

                        #[watch]
                        set_visible: model.requirements.cpu.as_ref()
                            .map(|cpu| cpu.frequency.is_some())
                            .unwrap_or_default(),

                        add_suffix = &gtk::Label {
                            add_css_class: "dim-label",

                            #[watch]
                            set_label?: model.requirements.cpu.as_ref()
                                .and_then(|cpu| {
                                    cpu.frequency.map(|frequency| {
                                        let frequency = pretty_frequency(frequency);

                                        // TODO: i18n
                                        format!("{:.2} {}", frequency.0, frequency.1)
                                    })
                                })
                                .as_deref()
                        }
                    }
                },

                adw::ExpanderRow {
                    set_title: "Graphics",

                    #[watch]
                    set_visible: model.requirements.gpu.is_some(),

                    add_suffix = &gtk::Label {
                        add_css_class: "dim-label",

                        #[watch]
                        set_label?: model.requirements.gpu.as_ref()
                            .map(|gpu| {
                                match &model.language {
                                    Some(lang) => gpu.model.translate(lang),
                                    None => gpu.model.default_translation()
                                }
                            })
                    },

                    add_row = &adw::ActionRow {
                        set_title: "VRAM",

                        add_suffix = &gtk::Label {
                            add_css_class: "dim-label",

                            #[watch]
                            set_label?: model.requirements.gpu.as_ref()
                                .and_then(|gpu| {
                                    gpu.vram.map(|vram| {
                                        let vram = pretty_bytes(vram);

                                        // TODO: i18n
                                        format!("{:.2} {}", vram.0, vram.1)
                                    })
                                })
                                .as_deref()
                        }
                    }
                },

                adw::ExpanderRow {
                    set_title: "Memory",

                    #[watch]
                    set_visible: model.requirements.ram.is_some(),

                    add_suffix = &gtk::Label {
                        add_css_class: "dim-label",

                        #[watch]
                        set_label?: model.requirements.ram.as_ref()
                            .map(|ram| {
                                let size = pretty_bytes(ram.size);

                                // TODO: i18n
                                format!("{:.2} {}", size.0, size.1)
                            })
                            .as_deref()
                    },

                    add_row = &adw::ActionRow {
                        set_title: "Frequency",

                        add_suffix = &gtk::Label {
                            add_css_class: "dim-label",

                            #[watch]
                            set_label?: model.requirements.ram.as_ref()
                                .and_then(|ram| {
                                    ram.frequency.map(|frequency| {
                                        let frequency = pretty_frequency(frequency);

                                        // TODO: i18n
                                        format!("{:.2} {}", frequency.0, frequency.1)
                                    })
                                })
                                .as_deref()
                        }
                    }
                },

                adw::ExpanderRow {
                    set_title: "Disk",

                    #[watch]
                    set_visible: model.requirements.disk.is_some(),

                    add_suffix = &gtk::Label {
                        add_css_class: "dim-label",

                        #[watch]
                        set_label?: model.requirements.disk.as_ref()
                            .map(|disk| {
                                let size = pretty_bytes(disk.size);

                                // TODO: i18n
                                format!("{:.2} {}", size.0, size.1)
                            })
                            .as_deref()
                    },

                    add_row = &adw::ActionRow {
                        set_title: "Type",

                        add_suffix = &gtk::Label {
                            add_css_class: "dim-label",

                            #[watch]
                            set_label?: model.requirements.disk.as_ref()
                                .and_then(|disk| {
                                    disk.disk_type.as_ref().map(|disk_type| {
                                        match disk_type {
                                            DiskType::Hdd  => "HDD",
                                            DiskType::Ssd  => "SSD",
                                            DiskType::Nvme => "NVMe"
                                        }
                                    })
                                })
                        }
                    }
                }
            }
        }
    }

    #[inline]
    async fn init(_init: Self::Init, root: Self::Root, _sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            requirements: HardwareRequirements::default(),
            language: None
        };

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            HardwareRequirementsSectionMsg::SetRequirements { requirements, language } => {
                self.requirements = requirements;
                self.language = language;
            }

            HardwareRequirementsSectionMsg::Clear => {
                self.requirements = HardwareRequirements::default();
                self.language = None;
            }
        }
    }
}
