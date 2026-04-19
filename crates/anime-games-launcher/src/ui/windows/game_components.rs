// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@vk.com>
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

#[derive(Debug)]
pub enum GameComponentsWindowInput {
    SetGame {
        variant: GameVariant,
        integration: Arc<GameIntegration>,
        layout: Box<[GameComponentsGroup]>
    },

    UpdateCurrentGameLayout
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameComponentsWindowOutput {
    ReloadGameInfo
}

#[derive(Debug, Clone)]
pub struct GameComponentsWindow {
    window: Option<adw::PreferencesDialog>,
    pages: Vec<adw::PreferencesPage>,
    entries: Vec<(String, adw::SwitchRow)>,

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
        _window = adw::PreferencesDialog {
            set_title: i18n!("game_components_title")
                .unwrap_or("Game components"),

            set_content_width: 800,
            set_content_height: 600,
            set_search_enabled: true,

            add_css_class?: consts::APP_DEBUG.then_some("devel")
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: AsyncComponentSender<Self>
    ) -> AsyncComponentParts<Self> {
        let mut model = Self {
            window: None,
            pages: Vec::with_capacity(1),
            entries: Vec::new(),

            game_variant: None,
            game_integration: None
        };

        let widgets = view_output!();

        model.window = Some(widgets._window.clone());

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
                if let Some(window) = self.window.clone() {
                    let lang = config::get().language().ok();

                    let pages = std::mem::take(&mut self.pages);

                    self.entries.clear();

                    let result = gtk::glib::spawn_future_local(async move {
                        for page in pages {
                            window.remove(&page);

                            drop(page);
                        }

                        let page_widget = adw::PreferencesPage::new();
                        let mut entries = Vec::new();

                        window.add(&page_widget);

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

                            page_widget.add(&group_widget);

                            for entry in group.entries() {
                                let entry_widget = adw::SwitchRow::new();

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

                                group_widget.add(&entry_widget);

                                entries.push((
                                    entry.name().to_string(),
                                    entry_widget
                                ));
                            }
                        }

                        (page_widget, entries)
                    }).await;

                    match result {
                        Ok((page_widget, entries)) => {
                            self.pages.push(page_widget);
                            self.entries = entries;
                        }

                        Err(err) => {
                            tracing::error!(?err, "failed to render game settings");

                            dialogs::error(
                                i18n!("failed_render_game_settings")
                                    .unwrap_or("Failed to render game settings"),
                                err.to_string()
                            );

                            return;
                        }
                    }

                    self.game_variant = Some(variant);
                    self.game_integration = Some(integration);
                }
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
                            if let Some(window) = &self.window {
                                window.close();
                            }
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
        }
    }
}
