use relm4::prelude::*;

use gtk::prelude::*;
use adw::prelude::*;

use crate::games::integrations::standards::addons::AddonsGroup;

use crate::ui::components::addon::addon_group::AddonsGroupComponent;
use crate::ui::components::game_card::CardInfo;

static mut WINDOW: Option<adw::ApplicationWindow> = None;

#[derive(Debug)]
pub struct GameAddonsManagerApp {
    pub addons_groups_widgets: Vec<AsyncController<AddonsGroupComponent>>,
    pub addons_groups_page: adw::PreferencesPage,

    pub info: CardInfo
}

#[derive(Debug, Clone)]
pub enum GameAddonsManagerAppMsg {
    SetGameInfo {
        info: CardInfo,
        addons: Vec<AddonsGroup>
    }
}

#[relm4::component(pub, async)]
impl SimpleAsyncComponent for GameAddonsManagerApp {
    type Init = adw::ApplicationWindow;
    type Input = GameAddonsManagerAppMsg;
    type Output = ();

    view! {
        window = adw::ApplicationWindow {
            set_default_size: (700, 560),
            set_title: Some("Game addons"),

            set_hide_on_close: true,
            set_modal: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    add_css_class: "flat"
                },

                #[local_ref]
                addons_groups_page -> adw::PreferencesPage,
            }
        }
    }

    async fn init(
        parent: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = Self {
            addons_groups_widgets: Vec::new(),
            addons_groups_page: adw::PreferencesPage::new(),

            info: CardInfo::default()
        };

        let addons_groups_page = &model.addons_groups_page;

        let widgets = view_output!();

        widgets.window.set_transient_for(Some(&parent));

        unsafe {
            WINDOW = Some(widgets.window.clone());
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(&mut self, msg: Self::Input, _sender: AsyncComponentSender<Self>) {
        match msg {
            GameAddonsManagerAppMsg::SetGameInfo { info, addons } => {
                self.info = info;

                for group in &self.addons_groups_widgets {
                    self.addons_groups_page.remove(group.widget());
                }

                self.addons_groups_widgets.clear();

                for group in addons {
                    let group = AddonsGroupComponent::builder()
                        .launch(group)
                        .detach();

                    self.addons_groups_page.add(group.widget());
                    self.addons_groups_widgets.push(group);
                }
            }
        }
    }
}
