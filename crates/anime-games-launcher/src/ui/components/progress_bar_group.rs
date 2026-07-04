// SPDX-License-Identifier: GPL-3.0-or-later
//
// anime-games-launcher
// Copyright (C) 2025 - 2026  Nikita Podvirnyi <krypt0nn@dawn.wine>
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

use adw::prelude::*;
use relm4::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum ProgressBarGroupValue {
    /// Don't display anything.
    None,

    /// Display spinner widget.
    Spinner,

    /// Display progress bar with given text and fraction.
    Progress {
        text: Option<String>,
        fraction: f64
    },

    /// Display checkmark icon.
    Finished
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProgressBarGroupMsg {
    SetValue(ProgressBarGroupValue),

    /// Set progress bar value to "spinner".
    ShowSpinner,

    /// Set progress bar value to "progress".
    SetProgress {
        text: Option<String>,
        fraction: f64
    },

    /// Set progress bar value to "finished".
    SetFinished,

    /// Set progress bar value to "none".
    Hide
}

/// A factory component that builds a `AdwPreferencesGroup` component with
/// `AdwActionRow`-s where each row can display a progress bar, a spinner,
/// a checkmark or nothing.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressBarGroup {
    /// Progress bar title.
    pub title: String,

    /// Progress bar subtitle.
    pub description: Option<String>,

    /// Progress bar value.
    pub value: ProgressBarGroupValue
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for ProgressBarGroup {
    type Init = Self;
    type Input = ProgressBarGroupMsg;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = adw::PreferencesGroup;

    view! {
        #[root]
        adw::ActionRow {
            set_title: &self.title,
            set_subtitle?: &self.description,

            add_suffix = &adw::Spinner {
                #[watch]
                set_visible: self.value == ProgressBarGroupValue::Spinner
            },

            add_suffix = &gtk::ProgressBar {
                set_valign: gtk::Align::Center,

                set_show_text: true,

                #[watch]
                set_visible: match &self.value {
                    ProgressBarGroupValue::Progress { text: None, fraction }
                        => *fraction > 0.0,

                    ProgressBarGroupValue::Progress { text: Some(text), fraction }
                        => !text.is_empty() || *fraction > 0.0,

                    _ => false
                },

                #[watch]
                set_text: match &self.value {
                    ProgressBarGroupValue::Progress { text, .. } => text.as_deref(),
                    _ => None
                },

                #[watch]
                set_fraction: match &self.value {
                    ProgressBarGroupValue::Progress { fraction, .. } => *fraction,
                    _ => 0.0
                },
            },

            add_suffix = &gtk::Image {
                #[watch]
                set_visible: self.value == ProgressBarGroupValue::Finished,

                set_icon_name: Some("emblem-ok-symbolic")
            }
        }
    }

    #[inline]
    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        init
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncFactorySender<Self>
    ) {
        match msg {
            ProgressBarGroupMsg::SetValue(value) => {
                self.value = value;
            }

            ProgressBarGroupMsg::ShowSpinner => {
                self.value = ProgressBarGroupValue::Spinner;
            }

            ProgressBarGroupMsg::SetProgress { text, fraction } => {
                self.value = ProgressBarGroupValue::Progress { text, fraction };
            }

            ProgressBarGroupMsg::SetFinished => {
                self.value = ProgressBarGroupValue::Finished;
            }

            ProgressBarGroupMsg::Hide => {
                self.value = ProgressBarGroupValue::None;
            }
        }
    }
}
