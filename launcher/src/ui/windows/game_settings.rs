// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2025  Nikita Podvirnyi <krypt0nn@vk.com>
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

use unic_langid::LanguageIdentifier;

use agl_games::engine::{
    GameIntegration,
    GameSettingsGroup,
    GameSettingsEntry,
    GameSettingsEntryFormat,
    GameSettingsEntryReactivity
};

use crate::ui::dialogs;

enum ParentWidget<'widget> {
    Group(&'widget adw::PreferencesGroup),
    Expandable(&'widget adw::ExpanderRow)
}

impl ParentWidget<'_> {
    pub fn add(&self, child: &impl IsA<gtk::Widget>) {
        match self {
            Self::Group(group) => group.add(child),
            Self::Expandable(row) => row.add_row(child)
        }
    }
}

fn render_entry(
    group_widget: ParentWidget<'_>,
    entry: &GameSettingsEntry,
    lang: Option<&LanguageIdentifier>,
    listener: relm4::Sender<GameSettingsWindowInput>
) {
    match entry.entry() {
        GameSettingsEntryFormat::Switch { value } => {
            let widget = adw::SwitchRow::new();

            let title = match lang {
                Some(lang) => entry.title().translate(lang),
                None => entry.title().default_translation()
            };

            widget.set_title(title);

            if let Some(description) = entry.description() {
                let description = match lang {
                    Some(lang) => description.translate(lang),
                    None => description.default_translation()
                };

                widget.set_subtitle(description);
            }

            widget.set_active(*value);

            if let Some(name) = entry.name().cloned() {
                let reactivity = entry.reactivity()
                    .copied()
                    .unwrap_or_default();

                widget.connect_active_notify(move |widget| {
                    listener.emit(GameSettingsWindowInput::SetBoolProperty {
                        name: name.clone(),
                        value: widget.is_active(),
                        reactivity
                    });
                });
            }

            group_widget.add(&widget);
        }

        GameSettingsEntryFormat::Text { value } => {
            let widget = adw::EntryRow::new();

            widget.set_show_apply_button(true);

            let title = match lang {
                Some(lang) => entry.title().translate(lang),
                None => entry.title().default_translation()
            };

            widget.set_title(title);

            if let Some(description) = entry.description() {
                let description = match lang {
                    Some(lang) => description.translate(lang),
                    None => description.default_translation()
                };

                widget.set_tooltip(description);
            }

            widget.set_text(value);

            if let Some(name) = entry.name().cloned() {
                let reactivity = entry.reactivity()
                    .copied()
                    .unwrap_or_default();

                widget.connect_apply(move |widget| {
                    listener.emit(GameSettingsWindowInput::SetStringProperty {
                        name: name.clone(),
                        value: widget.text().to_string(),
                        reactivity
                    });
                });
            }

            group_widget.add(&widget);
        }

        GameSettingsEntryFormat::Enum { values, selected } => {
            let widget = adw::ComboRow::new();

            let title = match lang {
                Some(lang) => entry.title().translate(lang),
                None => entry.title().default_translation()
            };

            widget.set_title(title);

            if let Some(description) = entry.description() {
                let description = match lang {
                    Some(lang) => description.translate(lang),
                    None => description.default_translation()
                };

                widget.set_tooltip(description);
            }

            let model = gtk::StringList::new(&[]);

            let mut selected_index = 0;

            for (i, (key, value)) in values.iter().enumerate() {
                let value = match lang {
                    Some(lang) => value.translate(lang),
                    None => value.default_translation()
                };

                model.append(value);

                if key == selected {
                    selected_index = i;
                }
            }

            widget.set_model(Some(&model));
            widget.set_selected(selected_index as u32);

            if let Some(name) = entry.name().cloned() {
                let reactivity = entry.reactivity()
                    .copied()
                    .unwrap_or_default();

                let values = values.clone();

                widget.connect_selected_notify(move |widget| {
                    let selected = widget.selected();

                    if let Some((key, _)) = values.get(selected as usize) {
                        listener.emit(GameSettingsWindowInput::SetStringProperty {
                            name: name.clone(),
                            value: key.to_owned(),
                            reactivity
                        });
                    }
                });
            }

            group_widget.add(&widget);
        }

        GameSettingsEntryFormat::Expandable { entries } => {
            let widget = adw::ExpanderRow::new();

            let title = match lang {
                Some(lang) => entry.title().translate(lang),
                None => entry.title().default_translation()
            };

            widget.set_title(title);

            if let Some(description) = entry.description() {
                let description = match lang {
                    Some(lang) => description.translate(lang),
                    None => description.default_translation()
                };

                widget.set_subtitle(description);
            }

            for entry in entries {
                render_entry(
                    ParentWidget::Expandable(&widget),
                    entry,
                    lang,
                    listener.clone()
                );
            }

            group_widget.add(&widget);
        }
    }
}

#[derive(Debug)]
pub enum GameSettingsWindowInput {
    SetGame {
        layout: Vec<GameSettingsGroup>,
        language: Option<LanguageIdentifier>,
        integration: Arc<GameIntegration>
    },

    SetBoolProperty {
        name: String,
        value: bool,
        reactivity: GameSettingsEntryReactivity
    },

    SetStringProperty {
        name: String,
        value: String,
        reactivity: GameSettingsEntryReactivity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSettingsWindowOutput {
    ReloadSettingsWindow,
    ReloadGameStatus
}

#[derive(Debug, Clone)]
pub struct GameSettingsWindow {
    window: Option<adw::PreferencesDialog>,
    pages: Vec<adw::PreferencesPage>,
    integration: Option<Arc<GameIntegration>>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameSettingsWindow {
    type Init = ();
    type Input = GameSettingsWindowInput;
    type Output = GameSettingsWindowOutput;

    view! {
        #[root]
        _window = adw::PreferencesDialog {
            set_title: "Settings",

            set_content_width: 800,
            set_content_height: 600,
            set_search_enabled: true
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
            integration: None
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
        fn handle_reactivity(
            reactivity: &GameSettingsEntryReactivity,
            sender: AsyncComponentSender<GameSettingsWindow>
        ) {
            match reactivity {
                GameSettingsEntryReactivity::Relaxed => {
                    let _ = sender.output(GameSettingsWindowOutput::ReloadGameStatus);
                }

                GameSettingsEntryReactivity::Release => {
                    let _ = sender.output(GameSettingsWindowOutput::ReloadGameStatus);
                    let _ = sender.output(GameSettingsWindowOutput::ReloadSettingsWindow);
                }

                _ => ()
            }
        }

        match msg {
            GameSettingsWindowInput::SetGame {
                layout,
                language,
                integration
            } => {
                if let Some(window) = self.window.clone() {
                    let pages = self.pages.drain(..).collect::<Vec<_>>();

                    let page_widget = gtk::glib::spawn_future_local(async move {
                        for page in pages {
                            window.remove(&page);

                            drop(page);
                        }

                        let page_widget = adw::PreferencesPage::new();

                        window.add(&page_widget);

                        for group in layout {
                            let group_widget = adw::PreferencesGroup::new();

                            if let Some(title) = group.title() {
                                let title = match language.as_ref() {
                                    Some(lang) => title.translate(lang),
                                    None => title.default_translation()
                                };

                                group_widget.set_title(title);
                            }

                            if let Some(description) = group.description() {
                                let description = match language.as_ref() {
                                    Some(lang) => description.translate(lang),
                                    None => description.default_translation()
                                };

                                group_widget.set_description(Some(description));
                            }

                            page_widget.add(&group_widget);

                            for entry in group.entries() {
                                render_entry(
                                    ParentWidget::Group(&group_widget),
                                    entry,
                                    language.as_ref(),
                                    sender.input_sender().clone()
                                );
                            }
                        }

                        page_widget
                    }).await;

                    match page_widget {
                        Ok(page_widget) => self.pages.push(page_widget),

                        Err(err) => {
                            tracing::error!(?err, "failed to render game settings");

                            dialogs::error("Failed to render game settings", err.to_string());

                            return;
                        }
                    }

                    self.integration = Some(integration);
                }
            }

            GameSettingsWindowInput::SetBoolProperty {
                name,
                value,
                reactivity
            } => {
                if let Some(integration) = &self.integration {
                    if let Err(err) = integration.set_property(name, value) {
                        tracing::error!(?err, "failed to set game property value");

                        dialogs::error("Failed to set game property value", err.to_string());

                        return;
                    }

                    handle_reactivity(&reactivity, sender);
                }
            }

            GameSettingsWindowInput::SetStringProperty {
                name,
                value,
                reactivity
            } => {
                if let Some(integration) = &self.integration {
                    if let Err(err) = integration.set_property(name, value) {
                        tracing::error!(?err, "failed to set game property value");

                        dialogs::error("Failed to set game property value", err.to_string());

                        return;
                    }

                    handle_reactivity(&reactivity, sender);
                }
            }
        }
    }
}
