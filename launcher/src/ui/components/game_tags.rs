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

use adw::prelude::*;
use relm4::prelude::*;

use agl_games::manifest::GameTag;

use crate::i18n;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameTagFactory(GameTag);

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for GameTagFactory {
    type Init = GameTag;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        gtk::Overlay {
            set_align: gtk::Align::Start,

            set_tooltip?: match self.0 {
                GameTag::Gambling          => i18n!("game_tag_gambling_description").map(String::from),
                GameTag::Payments          => i18n!("game_tag_payments_description").map(String::from),
                GameTag::GraphicViolence   => i18n!("game_tag_graphic_violence_description").map(String::from),
                GameTag::Cooperative       => i18n!("game_tag_cooperative_description").map(String::from),
                GameTag::Social            => i18n!("game_tag_social_description").map(String::from),
                GameTag::Controller        => i18n!("game_tag_controller_description").map(String::from),
                GameTag::PerformanceIssues => i18n!("game_tag_performance_issues_description").map(String::from),
                GameTag::AntiCheat         => i18n!("game_tag_anti_cheat_description").map(String::from),
                GameTag::Workarounds       => i18n!("game_tag_workarounds_description").map(String::from),
            }.as_deref(),

            gtk::Frame {
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    set_spacing: 4,
                    set_margin_all: 4,

                    gtk::Image {
                        set_icon_name: match self.0 {
                            GameTag::Gambling          => Some("dice3-symbolic"),
                            GameTag::Payments          => Some("money-symbolic"),
                            GameTag::GraphicViolence   => Some("violence-symbolic"),
                            GameTag::Cooperative       => Some("system-users-symbolic"),
                            GameTag::Social            => Some("mail-unread-symbolic"),
                            GameTag::Controller        => Some("input-gaming-symbolic"),
                            GameTag::PerformanceIssues => Some("speedometer4-symbolic"),
                            GameTag::AntiCheat         => Some("background-app-ghost-symbolic"),
                            GameTag::Workarounds       => Some("test-symbolic")
                        }
                    },

                    gtk::Label {
                        set_label: &match self.0 {
                            GameTag::Gambling => i18n!("game_tag_gambling_title")
                                .unwrap_or("Gambling")
                                .to_string(),

                            GameTag::Payments => i18n!("game_tag_payments_title")
                                .unwrap_or("Payments")
                                .to_string(),

                            GameTag::GraphicViolence => i18n!("game_tag_graphic_violence_title")
                                .unwrap_or("Graphic Violence")
                                .to_string(),

                            GameTag::Cooperative => i18n!("game_tag_cooperative_title")
                                .unwrap_or("Cooperative")
                                .to_string(),

                            GameTag::Social => i18n!("game_tag_social_title")
                                .unwrap_or("Social")
                                .to_string(),

                            GameTag::Controller => i18n!("game_tag_controller_title")
                                .unwrap_or("Controller")
                                .to_string(),

                            GameTag::PerformanceIssues => i18n!("game_tag_performance_issues_title")
                                .unwrap_or("Performance Issues")
                                .to_string(),

                            GameTag::AntiCheat => i18n!("game_tag_anti_cheat_title")
                                .unwrap_or("Anti-cheat")
                                .to_string(),

                            GameTag::Workarounds => i18n!("game_tag_workarounds_title")
                                .unwrap_or("Workarounds")
                                .to_string()
                        }
                    }
                }
            }
        }
    }

    #[inline]
    async fn init_model(
        tag: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>
    ) -> Self {
        Self(tag)
    }
}
