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

use agl_locale::unic_langid::LanguageIdentifier;

use agl_games::api::{
    GameVariant,
    GameIntegration,
    GameSettingsEntry,
    GameSettingsEntryFormat,
    GameSettingsEntryReactivity,
    GameSettingsGroup
};

use crate::{consts, config, i18n};
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

            if let Some(name) = entry.name() && *consts::APP_DEBUG {
                widget.set_tooltip(name);
            }

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

                if let Some(name) = entry.name() && *consts::APP_DEBUG {
                    widget.set_tooltip(&format!("[{name}] {description}"));
                } else {
                    widget.set_tooltip(description);
                }
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

        GameSettingsEntryFormat::SecretText { value } => {
            let widget = adw::PasswordEntryRow::new();

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

                if let Some(name) = entry.name() && *consts::APP_DEBUG {
                    widget.set_tooltip(&format!("[{name}] {description}"));
                } else {
                    widget.set_tooltip(description);
                }
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

        GameSettingsEntryFormat::Number { min, max, step, value } => {
            let min = (*min).unwrap_or(f64::MIN);
            let max = (*max).unwrap_or(f64::MAX);
            let step = (*step).unwrap_or(if max < 1.0 { max / 10.0 } else { 1.0 });

            let adjustment = gtk::Adjustment::new(
                *value,
                min,
                max,
                step,
                0.0,
                0.0
            );

            fn digits_num(value: f64) -> u32 {
                if !value.is_finite() {
                    return 0;
                }

                let s = format!("{:.6}", value.abs());

                let parts: Vec<&str> = s.split('.').collect();

                if parts.len() != 2 {
                    return 0;
                }

                let frac = parts[1].trim_end_matches('0');

                if frac.is_empty() {
                    0
                } else {
                    frac.len() as u32
                }
            }

            let widget = adw::SpinRow::new(
                Some(&adjustment),
                step,
                digits_num(step)
            );

            if let Some(name) = entry.name() && *consts::APP_DEBUG {
                widget.set_tooltip(name);
            }

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

            if let Some(name) = entry.name().cloned() {
                let reactivity = entry.reactivity()
                    .copied()
                    .unwrap_or_default();

                widget.connect_changed(move |widget| {
                    listener.emit(GameSettingsWindowInput::SetNumberProperty {
                        name: name.clone(),
                        value: widget.value(),
                        reactivity
                    });
                });
            }

            group_widget.add(&widget);
        }

        GameSettingsEntryFormat::Enum { values, selected } => {
            let widget = adw::ComboRow::new();

            if let Some(name) = entry.name() && *consts::APP_DEBUG {
                widget.set_tooltip(name);
            }

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

            if values.len() >= 10 {
                widget.set_enable_search(true);
                widget.set_search_match_mode(gtk::StringFilterMatchMode::Substring);

                let expression = gtk::PropertyExpression::new(
                    gtk::StringObject::static_type(),
                    None::<gtk::Expression>,
                    "string"
                );

                widget.set_expression(Some(expression));
            }

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

        GameSettingsEntryFormat::Selector { values, selected } => {
            let row = adw::ActionRow::new();
            let selector = adw::ToggleGroup::new();

            selector.set_valign(gtk::Align::Center);

            row.add_suffix(&selector);

            if let Some(name) = entry.name() && *consts::APP_DEBUG {
                row.set_tooltip(name);
                selector.set_tooltip(name);
            }

            let title = match lang {
                Some(lang) => entry.title().translate(lang),
                None => entry.title().default_translation()
            };

            row.set_title(title);

            if let Some(description) = entry.description() {
                let description = match lang {
                    Some(lang) => description.translate(lang),
                    None => description.default_translation()
                };

                row.set_subtitle(description);
            }

            let mut selected_index = 0;

            for (i, (key, value)) in values.iter().enumerate() {
                let value = match lang {
                    Some(lang) => value.translate(lang),
                    None => value.default_translation()
                };

                let toggle = adw::Toggle::new();

                toggle.set_label(Some(value));

                selector.add(toggle);

                if key == selected {
                    selected_index = i;
                }
            }

            selector.set_active(selected_index as u32);

            if let Some(name) = entry.name().cloned() {
                let reactivity = entry.reactivity()
                    .copied()
                    .unwrap_or_default();

                let values = values.clone();

                selector.connect_active_notify(move |widget| {
                    let selected = widget.active();

                    if let Some((key, _)) = values.get(selected as usize) {
                        listener.emit(GameSettingsWindowInput::SetStringProperty {
                            name: name.clone(),
                            value: key.to_owned(),
                            reactivity
                        });
                    }
                });
            }

            group_widget.add(&row);
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
        integration: Arc<GameIntegration>,
        variant: GameVariant,
        layout: Box<[GameSettingsGroup]>
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
    },

    SetNumberProperty {
        name: String,
        value: f64,
        reactivity: GameSettingsEntryReactivity
    },

    UpdateCurrentGameLayout
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSettingsWindowOutput {
    ReloadGameInfo
}

#[derive(Debug, Clone)]
pub struct GameSettingsWindow {
    window: adw::PreferencesDialog,
    page: adw::PreferencesPage,

    groups: Vec<adw::PreferencesGroup>,

    game_integration: Option<Arc<GameIntegration>>,
    game_variant: Option<GameVariant>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameSettingsWindow {
    type Init = ();
    type Input = GameSettingsWindowInput;
    type Output = GameSettingsWindowOutput;

    view! {
        #[root]
        adw::PreferencesDialog {
            set_title: i18n!("settings").unwrap_or("Settings"),

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

            // Some random capacity value I took from my head.
            groups: Vec::with_capacity(2),

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
        fn handle_reactivity(
            reactivity: &GameSettingsEntryReactivity,
            sender: AsyncComponentSender<GameSettingsWindow>
        ) {
            match reactivity {
                GameSettingsEntryReactivity::Relaxed => {
                    let _ = sender.output(GameSettingsWindowOutput::ReloadGameInfo);
                }

                GameSettingsEntryReactivity::Release => {
                    let _ = sender.output(GameSettingsWindowOutput::ReloadGameInfo);

                    sender.input(GameSettingsWindowInput::UpdateCurrentGameLayout);
                }

                _ => ()
            }
        }

        match msg {
            GameSettingsWindowInput::SetGame {
                integration,
                variant,
                layout
            } => {
                let lang = config::get().language().ok();

                let page = self.page.clone();

                self.game_integration = Some(integration);
                self.game_variant = Some(variant);

                let groups = std::mem::take(&mut self.groups);

                let groups = gtk::glib::spawn_future_local(async move {
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
                            render_entry(
                                ParentWidget::Group(&group_widget),
                                entry,
                                lang.as_ref(),
                                sender.input_sender().clone()
                            );
                        }

                        groups.push(group_widget);
                    }

                    groups
                }).await;

                match groups {
                    // Store groups *only if we've rendered them*. Otherwise we
                    // will keep the window *blank*.
                    Ok(groups) => self.groups = groups,

                    Err(err) => {
                        tracing::error!(?err, "failed to render game settings");

                        dialogs::error(
                            i18n!("failed_render_game_settings")
                                .unwrap_or("Failed to render game settings"),
                            err.to_string()
                        );
                    }
                }
            }

            GameSettingsWindowInput::SetBoolProperty {
                name,
                value,
                reactivity
            } => {
                if let Some(integration) = &self.game_integration {
                    if let Err(err) = integration.set_property(name, value) {
                        tracing::error!(?err, "failed to set game property value");

                        dialogs::error(
                            i18n!("failed_set_game_property")
                                .unwrap_or("Failed to set game property value"),
                            err.to_string()
                        );

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
                if let Some(integration) = &self.game_integration {
                    if let Err(err) = integration.set_property(name, value) {
                        tracing::error!(?err, "failed to set game property value");

                        dialogs::error(
                            i18n!("failed_set_game_property")
                                .unwrap_or("Failed to set game property value"),
                            err.to_string()
                        );

                        return;
                    }

                    handle_reactivity(&reactivity, sender);
                }
            }

            GameSettingsWindowInput::SetNumberProperty {
                name,
                value,
                reactivity
            } => {
                if let Some(integration) = &self.game_integration {
                    if let Err(err) = integration.set_property(name, value) {
                        tracing::error!(?err, "failed to set game property value");

                        dialogs::error(
                            i18n!("failed_set_game_property")
                                .unwrap_or("Failed to set game property value"),
                            err.to_string()
                        );

                        return;
                    }

                    handle_reactivity(&reactivity, sender);
                }
            }

            GameSettingsWindowInput::UpdateCurrentGameLayout => {
                if let Some(variant) = &self.game_variant &&
                    let Some(integration) = &self.game_integration
                {
                    match integration.get_settings_layout(variant) {
                        Ok(Some(layout)) => {
                            sender.input(GameSettingsWindowInput::SetGame {
                                variant: variant.clone(),
                                integration: integration.clone(),
                                layout
                            });
                        }

                        Ok(None) => {
                            self.window.close();
                        }

                        Err(err) => {
                            tracing::error!(?err, "failed to update game settings layout");

                            dialogs::error(
                                i18n!("failed_update_game_settings_layout")
                                    .unwrap_or("Failed to update game settings layout"),
                                err.to_string()
                            );
                        }
                    }
                }
            }
        }
    }
}
