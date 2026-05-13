// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2026  Nikita Podvirnyi <krypt0nn@vk.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashMap;
use std::sync::Arc;

use relm4::prelude::*;
use adw::prelude::*;

use agl_games::api::{
    GameVariant,
    GameIntegration,
    GameComponentsGroup,
    GameComponentsEntryValue,
    GameComponentEntryValueStatus
};

use crate::{consts, config, i18n};
use crate::ui::dialogs;

#[derive(Debug, Clone, PartialEq)]
struct ComponentState {
    pub checkbox_widget: gtk::CheckButton,

    pub prev_state: bool,
    pub curr_state: bool,

    pub is_locked: bool,
    pub entry_values: Box<[GameComponentsEntryValue]>
}

#[derive(Debug)]
pub enum GameComponentsWindowInput {
    SetGame {
        variant: GameVariant,
        integration: Arc<GameIntegration>,
        layout: Box<[GameComponentsGroup]>
    },

    UpdateCurrentGameLayout,

    SetComponentState {
        component: String,
        enabled: bool
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameComponentsWindowOutput {
    ReloadGameInfo
}

#[derive(Debug, Clone)]
pub struct GameComponentsWindow {
    window: adw::PreferencesDialog,
    page: adw::PreferencesPage,
    footer_group: adw::PreferencesGroup,

    groups: Vec<adw::PreferencesGroup>,
    entries: HashMap<String, ComponentState>,

    game_variant: Option<GameVariant>,
    game_integration: Option<Arc<GameIntegration>>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameComponentsWindow {
    type Init = ();
    type Input = GameComponentsWindowInput;
    type Output = GameComponentsWindowOutput;

    view! {
        #[root]
        adw::PreferencesDialog {
            set_title: i18n!("game_components_title")
                .unwrap_or("Game components"),

            set_content_width: 800,
            set_content_height: 600,
            set_search_enabled: true,

            add_css_class?: consts::APP_DEBUG.then_some("devel"),

            #[local_ref]
            add = page -> adw::PreferencesPage {
                set_description: i18n!("game_components_description")
                    .unwrap_or(""),

                #[local_ref]
                footer_group -> adw::PreferencesGroup {
                    adw::ButtonRow {
                        #[watch]
                        set_visible: model.entries.values()
                            .any(|component| component.prev_state != component.curr_state),

                        add_css_class: "suggested-action",

                        set_start_icon_name: Some("document-save-symbolic"),

                        set_title: i18n!("game_components_apply_changes_button_title")
                            .unwrap_or("Apply components changes")
                    },

                    adw::ButtonRow {
                        #[watch]
                        set_visible: model.entries.values()
                            .all(|component| component.prev_state == component.curr_state),

                        add_css_class: "destructive-action",

                        set_start_icon_name: Some("user-trash-symbolic"),

                        set_title: i18n!("game_components_uninstall_all_button_title")
                            .unwrap_or("Uninstall all"),

                        set_tooltip: i18n!("game_components_uninstall_all_button_description")
                            .unwrap_or("")
                    }
                }
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            window: root.clone(),
            page: adw::PreferencesPage::new(),
            footer_group: adw::PreferencesGroup::new(),

            // Some random capacity values I took from my head.
            groups: Vec::with_capacity(2),
            entries: HashMap::with_capacity(5),

            game_variant: None,
            game_integration: None
        };

        let page = &model.page;
        let footer_group = &model.footer_group;

        let widgets = view_output!();

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        sender: AsyncComponentSender<Self>
    ) {
        match msg {
            GameComponentsWindowInput::SetGame {
                variant,
                integration,
                layout
            } => {
                let lang = config::get().language().ok();

                let page = self.page.clone();
                let footer_group = self.footer_group.clone();

                let groups = std::mem::take(&mut self.groups);
                let mut entries = HashMap::new();

                self.entries.clear();

                // Prepare components hashmap for info that needs time to fetch.
                for group in &layout {
                    for entry in group.entries() {
                        let is_enabled = match integration.get_component_enabled(
                            &variant,
                            entry.name()
                        ) {
                            // Theoretically we should never receive `None` here
                            // since the games API guarantees get/set_enabled
                            // to be defined, but we will fallback to "enabled"
                            // for every component just in case.
                            Ok(result) => result.unwrap_or(true),

                            Err(err) => {
                                tracing::error!(?err, "failed to render game components");

                                dialogs::error(
                                    i18n!("failed_render_game_components")
                                        .unwrap_or("Failed to render game components"),
                                    err.to_string()
                                );

                                return;
                            }
                        };

                        entries.insert(
                            entry.name().to_string(),
                            ComponentState {
                                checkbox_widget: gtk::CheckButton::new(),

                                prev_state: is_enabled,
                                curr_state: is_enabled,

                                is_locked: entry.is_locked(),

                                entry_values: entry.values()
                                    .to_vec()
                                    .into_boxed_slice()
                            }
                        );
                    }
                }

                // Render the GUI widgets for components.
                let result = gtk::glib::spawn_future_local(async move {
                    for group in groups {
                        page.remove(&group);
                    }

                    page.remove(&footer_group);

                    let mut groups = Vec::with_capacity(layout.len());

                    for group in layout {
                        let group_widget = adw::PreferencesGroup::new();

                        if let Some(title) = group.title() {
                            let title = match lang.as_ref() {
                                Some(lang) => title.translate(lang),
                                None => title.default_translation()
                            };

                            group_widget.set_title(title);
                        }

                        if let Some(description) = group.description() {
                            let description = match lang.as_ref() {
                                Some(lang) => description.translate(lang),
                                None => description.default_translation()
                            };

                            group_widget.set_description(Some(description));
                        }

                        page.add(&group_widget);

                        for entry in group.entries() {
                            if let Some(component_state) = entries.get(entry.name()) {
                                let component_name = entry.name().to_string();

                                let entry_widget = adw::ExpanderRow::new();

                                if *consts::APP_DEBUG {
                                    entry_widget.set_tooltip(entry.name());
                                }

                                let title = match lang.as_ref() {
                                    Some(lang) => entry.title().translate(lang),
                                    None => entry.title().default_translation()
                                };

                                entry_widget.set_title(title);

                                if let Some(description) = entry.description() {
                                    let description = match lang.as_ref() {
                                        Some(lang) => description.translate(lang),
                                        None => description.default_translation()
                                    };

                                    entry_widget.set_subtitle(description);
                                }

                                // Render component values.
                                if component_state.entry_values.is_empty() {
                                    entry_widget.set_enable_expansion(false);
                                }

                                else {
                                    for value in &component_state.entry_values {
                                        let value_row = adw::ActionRow::new();

                                        let title = match lang.as_ref() {
                                            Some(lang) => value.title().translate(lang),
                                            None => value.title().default_translation()
                                        };

                                        value_row.set_title(title);

                                        if let Some(description) = value.description() {
                                            let description = match lang.as_ref() {
                                                Some(lang) => description.translate(lang),
                                                None => description.default_translation()
                                            };

                                            value_row.set_subtitle(description);
                                        }

                                        let value_label = match lang.as_ref() {
                                            Some(lang) => value.value().translate(lang),
                                            None => value.value().default_translation()
                                        };

                                        let value_widget = gtk::Label::new(Some(value_label));

                                        match value.status() {
                                            GameComponentEntryValueStatus::Normal => (),

                                            GameComponentEntryValueStatus::Warning => {
                                                value_widget.add_css_class("warning");
                                            }

                                            GameComponentEntryValueStatus::Danger => {
                                                value_widget.add_css_class("error");
                                            }

                                            GameComponentEntryValueStatus::Success => {
                                                value_widget.add_css_class("success");
                                            }
                                        }

                                        value_row.add_suffix(&value_widget);

                                        entry_widget.add_row(&value_row);
                                    }
                                }

                                // Render checkbox & uninstall button OR a lock
                                // icon for locked component.
                                if component_state.is_locked {
                                    let icon_widget = gtk::Image::from_icon_name("system-lock-screen-symbolic");

                                    icon_widget.set_width_request(26);

                                    entry_widget.add_prefix(&icon_widget);
                                }

                                else {
                                    component_state.checkbox_widget.set_active(component_state.curr_state);

                                    {
                                        let sender = sender.clone();
                                        let component_name = component_name.clone();

                                        component_state.checkbox_widget.connect_toggled(move |checkbox| {
                                            sender.input(GameComponentsWindowInput::SetComponentState {
                                                component: component_name.clone(),
                                                enabled: checkbox.is_active()
                                            });
                                        });
                                    }

                                    entry_widget.add_prefix(&component_state.checkbox_widget);
                                }

                                group_widget.add(&entry_widget);
                            }
                        }

                        groups.push(group_widget);
                    }

                    page.add(&footer_group);

                    (groups, entries)
                }).await;

                match result {
                    // Store groups and entries *only if we've rendered them*.
                    // Otherwise we will keep the window *blank*.
                    Ok((groups, entries)) => {
                        self.groups = groups;
                        self.entries = entries;
                    }

                    Err(err) => {
                        tracing::error!(?err, "failed to render game components");

                        dialogs::error(
                            i18n!("failed_render_game_components")
                                .unwrap_or("Failed to render game components"),
                            err.to_string()
                        );

                        return;
                    }
                }

                // Store game variant and integration object in any case so that
                // the window will be able to refresh it and try to render the
                // components again.
                self.game_variant = Some(variant);
                self.game_integration = Some(integration);
            }

            GameComponentsWindowInput::UpdateCurrentGameLayout => {
                if let Some(variant) = &self.game_variant &&
                    let Some(integration) = &self.game_integration
                {
                    match integration.get_components_layout(variant) {
                        Ok(Some(layout)) => {
                            sender.input(GameComponentsWindowInput::SetGame {
                                variant: variant.clone(),
                                integration: integration.clone(),
                                layout
                            });
                        }

                        Ok(None) => {
                            self.window.close();
                        }

                        Err(err) => {
                            tracing::error!(?err, "failed to update game components layout");

                            dialogs::error(
                                i18n!("failed_update_game_components_layout")
                                    .unwrap_or("Failed to update game components layout"),
                                err.to_string()
                            );
                        }
                    }
                }
            }

            GameComponentsWindowInput::SetComponentState {
                component,
                enabled
            } => {
                if let Some(component_state) = self.entries.get_mut(&component) {
                    component_state.curr_state = enabled;

                    component_state.checkbox_widget.set_active(enabled);
                }
            }
        }
    }
}
