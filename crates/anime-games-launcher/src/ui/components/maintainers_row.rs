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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaintainersRowFactory {
    pub name: String,
    pub link: Option<String>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaintainersRowFactoryMsg {
    Clicked
}

#[relm4::factory(pub, async)]
impl AsyncFactoryComponent for MaintainersRowFactory {
    type Init = String;
    type Input = MaintainersRowFactoryMsg;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = adw::ExpanderRow;

    view! {
        #[root]
        adw::ActionRow {
            set_title: &self.name,
            set_subtitle?: &self.link,

            set_activatable: true,

            connect_activated => MaintainersRowFactoryMsg::Clicked
        }
    }

    async fn init_model(
        init: Self::Init,
        _index: &DynamicIndex,
        _sender: AsyncFactorySender<Self>,
    ) -> Self {
        let name = init.trim();

        // Name <link>
        //
        // TODO: Technically "na<>me" <link> is also appropriate syntax, so this
        //       should be supported as well.
        if let Some(name) = name.strip_suffix('>') {
            let (name, link) = name.split_once('<')
                .unwrap_or((name, ""));

            Self {
                name: name.to_string(),
                link: Some(link.to_string())
            }
        }

        else {
            Self {
                name: name.to_string(),
                link: None
            }
        }
    }

    async fn update(
        &mut self,
        msg: Self::Input,
        _sender: AsyncFactorySender<Self>
    ) {
        match msg {
            MaintainersRowFactoryMsg::Clicked => {
                if let Some(link) = self.link.clone() {
                    // TODO: more strict rules.
                    let uri = if link.contains('@') {
                        format!("mailto:{link}")
                    } else if link.chars().all(|c| c.is_ascii_digit() || " +-".contains(c)) {
                        format!("tel:{}", link.replace([' ', '-'], ""))
                    } else {
                        link
                    };

                    let output = std::process::Command::new("xdg-open")
                        .arg(&uri)
                        .output();

                    if let Err(err) = output {
                        tracing::error!(
                            ?err,
                            name = ?self.name,
                            link = ?self.link,
                            ?uri,
                            "failed to open maintainer link"
                        );
                    }
                }
            }
        }
    }
}
