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

            set_tooltip: match self.0 {
                GameTag::Gambling          => "Game has scenes of gambling or has game mechanics related to gambling (wishes, banners, etc.)",
                GameTag::Payments          => "Game can accept real money for in-game content",
                GameTag::GraphicViolence   => "Game contains graphic violence",
                GameTag::Cooperative       => "Game has built-in multiplayer (cooperative) elements",
                GameTag::Social            => "Game has social features - online chat, VoIP, shared spaces, etc",
                GameTag::Controller        => "Game has controllers support",
                GameTag::PerformanceIssues => "Game is known to have bad performance, either globally across all the platforms or on the target platform specifically",
                GameTag::AntiCheat         => "Game has an anti-cheat, either server- or client-side. This tag doesn’t necessary mean that this anti-cheat doesn’t support the target platform",
                GameTag::Workarounds       => "Game cannot run on some platforms natively, but the integration package provides set of special utilities or game files modifications which make the game function. Note that this may violate its terms of service and result in taking actions on your account"
            },

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
                        set_label: match self.0 {
                            GameTag::Gambling          => "Gambling",
                            GameTag::Payments          => "Payments",
                            GameTag::GraphicViolence   => "Graphic Violence",
                            GameTag::Cooperative       => "Cooperative",
                            GameTag::Social            => "Social",
                            GameTag::Controller        => "Controller",
                            GameTag::PerformanceIssues => "Performance Issues",
                            GameTag::AntiCheat         => "Anti Cheat",
                            GameTag::Workarounds       => "Workarounds"
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
