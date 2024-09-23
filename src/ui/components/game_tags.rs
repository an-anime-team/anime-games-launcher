use adw::prelude::*;
use gtk::prelude::*;

use relm4::factory::*;
use relm4::prelude::*;

use crate::games::manifest::info::game_tag::*;

#[derive(Debug)]
pub struct GameTagFactory {
    pub tag: GameTag,
}

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
            set_tooltip: match self.tag {
                GameTag::Gambling => "Game has scenes of gambling or has game mechanics related to gambling (wishes, banners, etc.)",
                GameTag::Payments => "Game can accept real money for in-game content",
                GameTag::Violence => "Game contains graphic violence",
                GameTag::PerformanceIssues => "Game is known to have bad performance, either on any platform or on Linux specifically",
                GameTag::AntiCheat => "Game has an anti-cheat, either server- or client-side but may still work on Linux",
                GameTag::UnsupportedPlatform => "Game is not officially supported on Linux",
                GameTag::CompatibilityLayer => "Game is not runnable on Linux, but the integration package provides functionality to make the game runnable - this will likely be against TOS",
            },
            gtk::Frame {
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 4,
                    set_margin_all: 4,
                    gtk::Image {
                        set_icon_name: match self.tag {
                            GameTag::Gambling => Some("dice3-symbolic"),
                            GameTag::Payments => Some("money-symbolic"),
                            GameTag::Violence => Some("violence-symbolic"),
                            GameTag::PerformanceIssues => Some("speedometer4-symbolic"),
                            GameTag::AntiCheat => Some("background-app-ghost-symbolic"),
                            GameTag::UnsupportedPlatform => Some("cloud-disabled-symbolic"),
                            GameTag::CompatibilityLayer => Some("test-symbolic"),
                        },
                    },
                    gtk::Label {
                        set_label: match self.tag {
                            GameTag::Gambling => "Gambling",
                            GameTag::Payments => "Payments",
                            GameTag::Violence => "Violence",
                            GameTag::PerformanceIssues => "Performance Issues",
                            GameTag::AntiCheat => "Anti Cheat",
                            GameTag::UnsupportedPlatform => "Unsupported Platform",
                            GameTag::CompatibilityLayer => "Workarounds",
                        },
                    }
                }
            }
        }
    }

    async fn init_model(
        init: Self::Init,
        index: &DynamicIndex,
        sender: AsyncFactorySender<Self>,
    ) -> Self {
        Self { tag: init }
    }
}
