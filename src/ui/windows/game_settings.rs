use relm4::prelude::*;
use adw::prelude::*;

use tokio::sync::mpsc::UnboundedSender;
use unic_langid::LanguageIdentifier;

use crate::prelude::*;

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
    entry: GameSettingsEntry,
    lang: Option<&LanguageIdentifier>,
    listener: relm4::Sender<GameSettingsWindowInput>
) {
    match entry.entry {
        GameSettingsEntryFormat::Switch { value } => {
            let widget = adw::SwitchRow::new();

            let title = match lang {
                Some(lang) => entry.title.translate(lang),
                None => entry.title.default_translation()
            };

            widget.set_title(title);

            if let Some(description) = entry.description.as_ref() {
                let description = match lang {
                    Some(lang) => description.translate(lang),
                    None => description.default_translation()
                };

                widget.set_subtitle(description);
            }

            widget.set_active(value);

            if let Some(name) = entry.name {
                let reactivity = entry.reactivity.unwrap_or_default();

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

            let title = match lang {
                Some(lang) => entry.title.translate(lang),
                None => entry.title.default_translation()
            };

            widget.set_title(title);

            if let Some(description) = entry.description.as_ref() {
                let description = match lang {
                    Some(lang) => description.translate(lang),
                    None => description.default_translation()
                };

                widget.set_tooltip(description);
            }

            widget.set_text(&value);

            if let Some(name) = entry.name {
                widget.connect_changed(move |widget| {
                    let reactivity = entry.reactivity.unwrap_or_default();

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
                Some(lang) => entry.title.translate(lang),
                None => entry.title.default_translation()
            };

            widget.set_title(title);

            if let Some(description) = entry.description.as_ref() {
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

                if key == &selected {
                    selected_index = i;
                }
            }

            widget.set_model(Some(&model));
            widget.set_selected(selected_index as u32);

            if let Some(name) = entry.name {
                widget.connect_selected_notify(move |widget| {
                    let selected = widget.selected();

                    if let Some((key, _)) = values.get(selected as usize) {
                        let reactivity = entry.reactivity.unwrap_or_default();

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
                Some(lang) => entry.title.translate(lang),
                None => entry.title.default_translation()
            };

            widget.set_title(title);

            if let Some(description) = entry.description.as_ref() {
                let description = match lang {
                    Some(lang) => description.translate(lang),
                    None => description.default_translation()
                };

                widget.set_subtitle(description);
            }

            for entry in entries {
                render_entry(ParentWidget::Expandable(&widget), entry, lang, listener.clone());
            }

            group_widget.add(&widget);
        }
    }
}

#[derive(Debug)]
pub enum GameSettingsWindowInput {
    EmitPresent,

    RenderLayout {
        layout: Vec<GameSettingsGroup>,
        language: Option<LanguageIdentifier>,
        sender: UnboundedSender<SyncGameCommand>
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
    parent: adw::ApplicationWindow,
    sender: Option<UnboundedSender<SyncGameCommand>>,
    pages: Vec<adw::PreferencesPage>
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameSettingsWindow {
    type Init = adw::ApplicationWindow;
    type Input = GameSettingsWindowInput;
    type Output = GameSettingsWindowOutput;

    view! {
        #[root]
        window = adw::PreferencesDialog {
            set_title: "Settings",

            set_content_width: 800,
            set_content_height: 600,
            set_search_enabled: true
        }
    }

    async fn init(parent: Self::Init, root: Self::Root, _sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let mut model = Self {
            window: None,
            parent,
            sender: None,
            pages: Vec::with_capacity(1)
        };

        let widgets = view_output!();

        model.window = Some(widgets.window.clone());

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {
        fn handle_reactivity(reactivity: GameSettingsEntryReactivity, sender: AsyncComponentSender<GameSettingsWindow>) {
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
            GameSettingsWindowInput::EmitPresent => {
                if let Some(window) = self.window.as_ref() {
                    window.present(Some(&self.parent));
                }
            }

            GameSettingsWindowInput::RenderLayout { layout, language, sender: server_sender } => {
                self.sender = Some(server_sender);

                if let Some(window) = self.window.clone() {
                    let pages = self.pages.drain(..).collect::<Vec<_>>();

                    let page_widget = gtk::glib::spawn_future_local(async move {
                        for page in pages {
                            window.remove(&page);
                        }

                        let page_widget = adw::PreferencesPage::new();

                        window.add(&page_widget);

                        for group in layout {
                            let group_widget = adw::PreferencesGroup::new();

                            if let Some(title) = group.title.as_ref() {
                                let title = match language.as_ref() {
                                    Some(lang) => title.translate(lang),
                                    None => title.default_translation()
                                };

                                group_widget.set_title(title);
                            }

                            if let Some(description) = group.description.as_ref() {
                                let description = match language.as_ref() {
                                    Some(lang) => description.translate(lang),
                                    None => description.default_translation()
                                };

                                group_widget.set_description(Some(description));
                            }

                            page_widget.add(&group_widget);

                            for entry in group.entries {
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
                        Err(err) => tracing::error!(?err, "Failed to render game settings page")
                    }
                }
            }

            GameSettingsWindowInput::SetBoolProperty { name, value, reactivity } => {
                if let Some(server_sender) = self.sender.as_ref() {
                    let result = server_sender.send(SyncGameCommand::SetBoolProperty {
                        name,
                        value
                    });

                    if let Err(err) = result {
                        tracing::error!(?err, "Failed to set game property value");
                    }

                    handle_reactivity(reactivity, sender);
                }
            }

            GameSettingsWindowInput::SetStringProperty { name, value, reactivity } => {
                if let Some(server_sender) = self.sender.as_ref() {
                    let result = server_sender.send(SyncGameCommand::SetStringProperty {
                        name,
                        value
                    });

                    if let Err(err) = result {
                        tracing::error!(?err, "Failed to set game property value");
                    }

                    handle_reactivity(reactivity, sender);
                }
            }
        }
    }
}
