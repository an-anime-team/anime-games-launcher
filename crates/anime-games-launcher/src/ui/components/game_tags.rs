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

use adw::prelude::*;
use relm4::prelude::*;

use agl_games::manifest::GameTag;

use crate::i18n;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
                GameTag::FreeToPlay        => i18n!("game_tag_free_to_play_description").map(String::from),
                GameTag::Cooperative       => i18n!("game_tag_cooperative_description").map(String::from),
                GameTag::SocialFeatures    => i18n!("game_tag_social_features_description").map(String::from),
                GameTag::ControllerSupport => i18n!("game_tag_controller_support_description").map(String::from),
                GameTag::Gambling          => i18n!("game_tag_gambling_description").map(String::from),
                GameTag::InGamePurchases   => i18n!("game_tag_in_game_purchases_description").map(String::from),
                GameTag::GraphicViolence   => i18n!("game_tag_graphic_violence_description").map(String::from),
                GameTag::AdultContent      => i18n!("game_tag_adult_content_description").map(String::from),

                GameTag::Other(_) => None
            }.as_deref(),

            gtk::Frame {
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    set_spacing: 4,
                    set_margin_all: 4,

                    gtk::Image {
                        set_visible: !matches!(self.0, GameTag::Other(_)),

                        set_icon_name: match self.0 {
                            GameTag::FreeToPlay        => Some("social-network-symbolic"),
                            GameTag::Cooperative       => Some("system-users-symbolic"),
                            GameTag::SocialFeatures    => Some("mail-unread-symbolic"),
                            GameTag::ControllerSupport => Some("input-gaming-symbolic"),
                            GameTag::Gambling          => Some("dice3-symbolic"),
                            GameTag::InGamePurchases   => Some("money-symbolic"),
                            GameTag::GraphicViolence   => Some("violence-symbolic"),
                            GameTag::AdultContent      => Some("nudity-symbolic"),

                            GameTag::Other(_) => None
                        }
                    },

                    gtk::Label {
                        set_label: &match self.0 {
                            GameTag::FreeToPlay => i18n!("game_tag_free_to_play_title")
                                .unwrap_or("Free to play")
                                .to_string(),

                            GameTag::Cooperative => i18n!("game_tag_cooperative_title")
                                .unwrap_or("Cooperative")
                                .to_string(),

                            GameTag::SocialFeatures => i18n!("game_tag_social_features_title")
                                .unwrap_or("Social features")
                                .to_string(),

                            GameTag::ControllerSupport => i18n!("game_tag_controller_support_title")
                                .unwrap_or("Controller")
                                .to_string(),

                            GameTag::Gambling => i18n!("game_tag_gambling_title")
                                .unwrap_or("Gambling")
                                .to_string(),

                            GameTag::InGamePurchases => i18n!("game_tag_in_game_purchases_title")
                                .unwrap_or("In-game purchases")
                                .to_string(),

                            GameTag::GraphicViolence => i18n!("game_tag_graphic_violence_title")
                                .unwrap_or("Graphic violence")
                                .to_string(),

                            GameTag::AdultContent => i18n!("game_tag_adult_content_title")
                                .unwrap_or("Adult content")
                                .to_string(),

                            GameTag::Other(ref tag) => tag.replace(['-', '_'], " ")
                        }
                    }
                }
            }
        }
    }

    #[inline(always)]
    async fn init_model(
        tag: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>
    ) -> Self {
        Self(tag)
    }
}
