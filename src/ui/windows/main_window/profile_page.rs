use std::collections::HashMap;

use adw::prelude::*;
use relm4::prelude::*;

use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProfileFactoryComponent(Profile);

#[relm4::factory(async)]
impl AsyncFactoryComponent for ProfileFactoryComponent {
    type Init = Profile;
    type Input = ProfilePageMsg;
    type Output = ProfilePageMsg;
    type ParentWidget = adw::PreferencesGroup;
    type CommandOutput = ();

    view! {
        #[root]
        adw::ActionRow {
            #[watch]
            set_title: &self.0.name,

            #[watch]
            set_subtitle: &self.0.target.to_string(),

            set_activatable: true,

            add_suffix = &gtk::Button {
                set_align: gtk::Align::Center,

                add_css_class: "circular",
                set_icon_name: "user-trash-symbolic",

                set_tooltip_text: Some("Delete profile"),

                connect_clicked[sender, index] => move |_| {
                    let _ = sender.output(ProfilePageMsg::DeleteProfile(index.clone()));
                }
            },

            connect_activated[sender, index] => move |_| {
                let _ = sender.output(ProfilePageMsg::OpenProfileManagerDialog(index.clone()));
            },
        }
    }

    #[inline]
    async fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: AsyncFactorySender<Self>) -> Self {
        Self(init)
    }
}

#[derive(Debug, Clone)]
pub enum ProfilePageMsg {
    UpdateProfiles,
    OpenNewProfileDialog,
    OpenProfileManagerDialog(DynamicIndex),
    InsertProfile(Profile),
    DeleteProfile(DynamicIndex)
}

#[derive(Debug)]
pub struct ProfilePage {
    builder_window: AsyncController<ProfileBuilderWindow>,
    manager_window: AsyncController<ProfileManagerWindow>,

    profiles: AsyncFactoryVecDeque<ProfileFactoryComponent>,

    profile_hashes: HashMap<Hash, DynamicIndex>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for ProfilePage {
    type Init = ();
    type Input = ProfilePageMsg;
    type Output = ();

    view! {
        #[root]
        adw::PreferencesPage {
            add = &adw::PreferencesGroup {
                set_title: "Profiles",

                #[wrap(Some)]
                set_header_suffix = &gtk::Button {
                    set_align: gtk::Align::Center,

                    add_css_class: "flat",
                    set_icon_name: "list-add-symbolic",

                    set_tooltip_text: Some("Create new profile"),

                    connect_clicked => ProfilePageMsg::OpenNewProfileDialog
                },

                model.profiles.widget(),
            }
        }
    }

    async fn init(_init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {
            builder_window: ProfileBuilderWindow::builder()
                .launch(())
                .forward(sender.input_sender(), ProfilePageMsg::InsertProfile),

            manager_window: ProfileManagerWindow::builder()
                .launch(())
                .forward(sender.input_sender(), ProfilePageMsg::InsertProfile),

            profiles: AsyncFactoryVecDeque::builder()
                .launch_default()
                .forward(sender.input_sender(), std::convert::identity),

            profile_hashes: HashMap::new()
        };

        sender.input(ProfilePageMsg::UpdateProfiles);

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        match msg {
            ProfilePageMsg::UpdateProfiles => {
                let config = config::get();

                let store = ProfilesStore::new(config.profiles.store.path);

                match store.list().await {
                    Ok(profiles) => {
                        let mut guard = self.profiles.guard();

                        self.profile_hashes.clear();
                        guard.clear();

                        for profile in profiles {
                            self.profile_hashes.insert(*profile.id(), guard.push_back(profile));
                        }
                    }

                    Err(err) => tracing::error!(?err, "Failed to list profiles")
                }
            }

            ProfilePageMsg::InsertProfile(profile) => {
                let config = config::get();

                let store = ProfilesStore::new(config.profiles.store.path);

                match store.insert(&profile) {
                    Ok(_) => {
                        tracing::debug!(
                            id = profile.id().to_base32(),
                            name = profile.name,
                            "Updated profile"
                        );

                        sender.input(ProfilePageMsg::UpdateProfiles);
                    }

                    Err(err) => tracing::error!(
                        id = profile.id().to_base32(),
                        name = profile.name,
                        ?err,
                        "Failed to update profile"
                    )
                }
            }

            ProfilePageMsg::OpenNewProfileDialog => {
                // if let Some(window) = MAIN_WINDOW.lock().as_ref() {
                //     self.builder_window.widget().set_transient_for(Some(window));
                // }

                self.builder_window.emit(ProfileBuilderWindowInput::OpenWindow);
            }

            ProfilePageMsg::OpenProfileManagerDialog(index) => {
                let config = config::get();

                let store = ProfilesStore::new(config.profiles.store.path);

                let mut guard = self.profiles.guard();
                let index = index.current_index();

                if let Some(profile_component) = guard.get_mut(index) {
                    let id = profile_component.0.id().to_owned();

                    match store.read(&id) {
                        Ok(profile) => {
                            if profile_component.0 != profile {
                                profile_component.0 = profile.clone();

                                tracing::debug!(
                                    id = id.to_base32(),
                                    "Profile was updated on the disk. Updated UI element"
                                );
                            }

                            // if let Some(window) = MAIN_WINDOW.lock().as_ref() {
                            //     self.manager_window.widget().set_transient_for(Some(window));
                            // }

                            self.manager_window.emit(ProfileManagerWindowMsg::OpenWindow(profile));
                        }

                        Err(err) => {
                            tracing::warn!(
                                id = id.to_base32(),
                                ?err,
                                "Failed to open profile manager because it was deleted from the disk"
                            );

                            guard.remove(index);
                        }
                    }
                }
            }

            ProfilePageMsg::DeleteProfile(index) => todo!("Profile deletion is not implemented yet: {index:#?}")
        }
    }
}
