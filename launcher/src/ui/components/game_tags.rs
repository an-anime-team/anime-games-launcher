use adw::prelude::*;
use relm4::prelude::*;

use crate::games::manifest::info::game_tag::GameTag;

#[derive(Debug)]
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
                GameTag::Gambling            => "Game has scenes of gambling or has game mechanics related to gambling (wishes, banners, etc.)",
                GameTag::Payments            => "Game can accept real money for in-game content",
                GameTag::GraphicViolence     => "Game contains graphic violence",
                GameTag::PerformanceIssues   => "Game is known to have bad performance, either on any platform or on Linux specifically",
                GameTag::AntiCheat           => "Game has an anti-cheat, either server- or client-side but may still work on Linux",
                GameTag::UnsupportedPlatform => "Game is not officially supported on Linux",
                GameTag::Workarounds         => "Game is not runnable on Linux, but the integration package provides functionality to make the game runnable - this will likely be against TOS"
            },

            gtk::Frame {
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    set_spacing: 4,
                    set_margin_all: 4,

                    gtk::Image {
                        set_icon_name: match self.0 {
                            GameTag::Gambling            => Some("dice3-symbolic"),
                            GameTag::Payments            => Some("money-symbolic"),
                            GameTag::GraphicViolence     => Some("violence-symbolic"),
                            GameTag::PerformanceIssues   => Some("speedometer4-symbolic"),
                            GameTag::AntiCheat           => Some("background-app-ghost-symbolic"),
                            GameTag::UnsupportedPlatform => Some("cloud-disabled-symbolic"),
                            GameTag::Workarounds         => Some("test-symbolic")
                        }
                    },

                    gtk::Label {
                        set_label: match self.0 {
                            GameTag::Gambling            => "Gambling",
                            GameTag::Payments            => "Payments",
                            GameTag::GraphicViolence     => "Graphic Violence",
                            GameTag::PerformanceIssues   => "Performance Issues",
                            GameTag::AntiCheat           => "Anti Cheat",
                            GameTag::UnsupportedPlatform => "Unsupported Platform",
                            GameTag::Workarounds         => "Workarounds"
                        }
                    }
                }
            }
        }
    }

    #[inline]
    async fn init_model(tag: Self::Init, _index: &DynamicIndex, _sender: AsyncFactorySender<Self>) -> Self {
        Self(tag)
    }
}
