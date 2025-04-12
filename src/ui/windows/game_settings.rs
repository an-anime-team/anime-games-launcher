use relm4::prelude::*;
use adw::prelude::*;

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

fn render_entry(group_widget: ParentWidget<'_>, entry: GameSettingsEntry, lang: Option<&LanguageIdentifier>) {
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

            for item in values.values() {
                let item = match lang {
                    Some(lang) => item.translate(lang),
                    None => item.default_translation()
                };

                model.append(item);
            }

            widget.set_model(Some(&model));

            if let Some((k, _)) = values.keys().enumerate().find(|(_, k)| k == &&selected) {
                widget.set_selected(k as u32);
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
                render_entry(ParentWidget::Expandable(&widget), entry, lang);
            }

            group_widget.add(&widget);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameSettingsWindowInit {
    pub layout: Vec<GameSettingsGroup>,
    pub language: Option<LanguageIdentifier>
}

#[derive(Debug)]
pub enum GameSettingsWindowInput {

}

#[derive(Debug)]
pub enum GameSettingsWindowOutput {

}

#[derive(Debug)]
pub struct GameSettingsWindow {

}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameSettingsWindow {
    type Init = GameSettingsWindowInit;
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

    async fn init(init: Self::Init, root: Self::Root, sender: AsyncComponentSender<Self>) -> AsyncComponentParts<Self> {
        let model = Self {};

        let widgets = view_output!();

        let window = widgets.window.clone();

        gtk::glib::spawn_future_local(async move {
            let page_widget = adw::PreferencesPage::new();

            window.add(&page_widget);

            for group in init.layout {
                let group_widget = adw::PreferencesGroup::new();

                if let Some(title) = group.title.as_ref() {
                    let title = match init.language.as_ref() {
                        Some(lang) => title.translate(lang),
                        None => title.default_translation()
                    };

                    group_widget.set_title(title);
                }

                if let Some(description) = group.description.as_ref() {
                    let description = match init.language.as_ref() {
                        Some(lang) => description.translate(lang),
                        None => description.default_translation()
                    };

                    group_widget.set_description(Some(description));
                }

                page_widget.add(&group_widget);

                for entry in group.entries {
                    render_entry(ParentWidget::Group(&group_widget), entry, init.language.as_ref());
                }
            }
        });

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, sender: AsyncComponentSender<Self>) {

    }
}
