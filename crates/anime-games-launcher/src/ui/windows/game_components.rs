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
    GameComponentsGroup
};

use crate::{consts, config, i18n};
use crate::ui::dialogs;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ComponentState {
    pub checkbox_widget: gtk::CheckButton,

    pub prev_value: bool,
    pub curr_value: bool
}

#[derive(Debug)]
pub enum GameComponentsWindowInput {
    SetGame {
        variant: GameVariant,
        integration: Arc<GameIntegration>,
        layout: Box<[GameComponentsGroup]>
    },

    UpdateCurrentGameLayout,

    EmitSwitchComponentState(String)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameComponentsWindowOutput {
    ReloadGameInfo
}

#[derive(Debug, Clone)]
pub struct GameComponentsWindow {
    window: adw::PreferencesDialog,
    page: adw::PreferencesPage,

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
            add = page -> adw::PreferencesPage,
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

            // Some random capacity values I took from my head.
            groups: Vec::with_capacity(2),
            entries: HashMap::with_capacity(5),

            game_variant: None,
            game_integration: None
        };

        let page = &model.page;

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

                                prev_value: is_enabled,
                                curr_value: is_enabled
                            }
                        );
                    }
                }

                // Render the GUI widgets for components.
                let result = gtk::glib::spawn_future_local(async move {
                    for group in groups {
                        page.remove(&group);
                    }

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
                            if let Some(entry_state) = entries.get(entry.name()) {
                                let entry_widget = adw::ActionRow::new();

                                entry_state.checkbox_widget.set_sensitive(false);
                                entry_state.checkbox_widget.set_active(entry_state.curr_value);

                                entry_widget.add_prefix(&entry_state.checkbox_widget);

                                entry_widget.set_activatable(true);

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

                                let sender = sender.clone();
                                let component = entry.name().to_string();

                                entry_widget.connect_activated(move |_| {
                                    sender.input(GameComponentsWindowInput::EmitSwitchComponentState(
                                        component.clone()
                                    ));
                                });

                                group_widget.add(&entry_widget);
                            }
                        }

                        groups.push(group_widget);
                    }

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

            GameComponentsWindowInput::EmitSwitchComponentState(component) => {
                if let Some(component_state) = self.entries.get_mut(&component) {
                    component_state.curr_value = !component_state.curr_value;

                    component_state.checkbox_widget.set_active(component_state.curr_value);
                }
            }
        }
    }
}
