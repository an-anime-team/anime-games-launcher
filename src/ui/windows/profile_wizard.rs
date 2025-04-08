use adw::prelude::*;
use relm4::prelude::*;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileWizardWindowInput {
    OpenWindow,
    CloseWindow,

    EmitClick
}

#[derive(Debug)]
pub struct ProfileWizardWindow {
    window: Option<adw::Window>,
    name_entry_row: adw::EntryRow,
    platform_combo_row: adw::ComboRow
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for ProfileWizardWindow {
    type Init = ();
    type Input = ProfileWizardWindowInput;
    type Output = Profile;

    view! {
        #[root]
        window = adw::Window {
            set_size_request: (700, 560),
            set_title: Some("New profile"),

            set_hide_on_close: true,
            set_modal: true,

            add_css_class?: crate::APP_DEBUG.then_some("devel"),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    add_css_class: "flat"
                },

                adw::PreferencesPage {
                    set_title: "Profile",

                    add = &adw::PreferencesGroup {
                        #[local_ref]
                        name_entry_row -> adw::EntryRow {
                            set_title: "Profile name"
                        },

                        #[local_ref]
                        platform_combo_row -> adw::ComboRow {
                            set_title: "Platform",
                            set_subtitle: "Environment emulated by this profile",

                            set_model: Some(&{
                                let list = gtk::StringList::new(&[]);

                                for platform in TargetPlatform::list() {
                                    list.append(&platform.to_string());
                                }

                                list
                            })
                        }
                    },

                    add = &adw::PreferencesGroup {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,

                            set_spacing: 8,

                            gtk::Button {
                                add_css_class: "suggested-action",
                                add_css_class: "pill",

                                set_label: "Create",

                                connect_clicked => ProfileBuilderWindowInput::EmitClick
                            }
                        }
                    }
                }
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let mut model = Self {
            window: None,
            name_entry_row: adw::EntryRow::new(),
            platform_combo_row: adw::ComboRow::new()
        };

        let name_entry_row = &model.name_entry_row;
        let platform_combo_row = &model.platform_combo_row;

        let widgets = view_output!();

        model.window = Some(widgets.window.clone());

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            ProfileBuilderWindowInput::OpenWindow => {
                if let Some(window) = self.window.as_ref() {
                    let suggested_platform = match CURRENT_PLATFORM.as_ref() {
                        Some(current_platform) => {
                            let suggested_platform = current_platform.suggested_emulation();

                            TargetPlatform::list()
                                .iter()
                                .position(|platform| platform == &suggested_platform)
                                .unwrap_or_default() as u32
                        }

                        None => 0
                    };

                    self.name_entry_row.set_text("");
                    self.platform_combo_row.set_selected(suggested_platform);

                    window.present();
                }
            }

            ProfileBuilderWindowInput::CloseWindow => {
                if let Some(window) = self.window.as_ref() {
                    window.close();
                }
            }

            ProfileBuilderWindowInput::EmitClick => {
                let name = self.name_entry_row.text();
                let platform = self.platform_combo_row.selected();

                let name = if name.is_empty() {
                    "New profile"
                } else {
                    name.as_str()
                };

                let profile = match TargetPlatform::list().get(platform as usize) {
                    Some(TargetPlatform::X86_64_windows_native) => Profile::builder()
                        .with_name(name)
                        .with_target_platform(TargetPlatform::X86_64_windows_native)
                        .with_general(GeneralProfileSettings::Windows {
                            common: CommonGeneralProfileSettings::default(),
                            windows: WindowsGeneralProfileSettings::default()
                        })
                        .with_runtime(RuntimeProfileSettings::None)
                        .build(),

                    Some(TargetPlatform::X86_64_linux_native) => Profile::builder()
                        .with_name(name)
                        .with_target_platform(TargetPlatform::X86_64_linux_native)
                        .with_general(GeneralProfileSettings::Linux {
                            common: CommonGeneralProfileSettings::default(),
                            linux: LinuxGeneralProfileSettings::default()
                        })
                        .with_runtime(RuntimeProfileSettings::None)
                        .build(),

                    Some(TargetPlatform::X86_64_linux_wine32) => Profile::builder()
                        .with_name(name)
                        .with_target_platform(TargetPlatform::X86_64_linux_wine32)
                        .with_general(GeneralProfileSettings::Linux {
                            common: CommonGeneralProfileSettings::default(),
                            linux: LinuxGeneralProfileSettings::default()
                        })
                        .with_runtime(RuntimeProfileSettings::LinuxWine(LinuxWineProfileRuntimeSettings::default()))
                        .build(),

                    Some(TargetPlatform::X86_64_linux_wine64) => Profile::builder()
                        .with_name(name)
                        .with_target_platform(TargetPlatform::X86_64_linux_wine64)
                        .with_general(GeneralProfileSettings::Linux {
                            common: CommonGeneralProfileSettings::default(),
                            linux: LinuxGeneralProfileSettings::default()
                        })
                        .with_runtime(RuntimeProfileSettings::LinuxWine(LinuxWineProfileRuntimeSettings::default()))
                        .build(),

                    None => {
                        tracing::error!("Invalid target platform variant: {platform}");

                        return;
                    }
                };

                let _ = sender.output(profile);

                if let Some(window) = &self.window {
                    window.close();
                }
            }
        }
    }
}
