use std::collections::HashMap;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, calculate_server_permissions,
    components::{Category, ChannelButton, ChannelListContextMenu},
    user_permissions_query,
};

#[derive(PartialEq)]
pub struct ChannelList {
    pub server: Readable<v0::Server>,
}

impl Component for ChannelList {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Channels);
        let channels: Readable<HashMap<String, v0::Channel>> =
            radio.slice_current(|state| &state.channels).into_readable();

        let non_category_channels = use_memo({
            let server = self.server.clone();

            move || {
                let server = server.read();
                let categories = server.categories.clone().unwrap_or_default();

                let category_channels = categories
                    .iter()
                    .filter(|c| c.id != "default")
                    .flat_map(|cat| cat.channels.clone())
                    .collect::<Vec<_>>();

                let mut non_category_channels = server
                    .channels
                    .iter()
                    .filter(|&channel_id| !category_channels.contains(channel_id))
                    .cloned()
                    .collect::<Vec<_>>();

                if let Some(default_cat) = categories.iter().find(|c| c.id == "default") {
                    non_category_channels.sort_by_key(|id| {
                        default_cat
                            .channels
                            .iter()
                            .position(|c| c == id)
                            .unwrap_or(usize::MAX)
                    });
                };

                non_category_channels
            }
        });

        rect()
            .padding((0., 0., 8., 8.))
            .on_secondary_down({
                let server = self.server.clone();
                move |_| {
                    let server = server.read().clone();

                    spawn(async move {
                        let mut query = user_permissions_query(radio).server(server.clone());

                        let permissions = calculate_server_permissions(&mut query).await;

                        ContextMenu::open(Menu::new().key(&server.id).child(
                            ChannelListContextMenu {
                                server_id: server.id.clone(),
                                current_permissions: permissions,
                            },
                        ))
                    });
                }
            })
            .child(
                ScrollView::new().child(
                    rect()
                        .cross_align(Alignment::Center)
                        .color(0xff90909a)
                        .child(
                            rect().padding((0., 0., 4., 0.)).children(
                                non_category_channels
                                    .read()
                                    .iter()
                                    .cloned()
                                    .filter_map(|channel_id| {
                                        channels.read().get(&channel_id).cloned()
                                    })
                                    .map(|channel| {
                                        ChannelButton {
                                            channel: channel.into_readable(),
                                            server: self.server.clone(),
                                        }
                                        .into_element()
                                    }),
                            ),
                        )
                        .child(
                            rect().spacing(8.).children(
                                self.server
                                    .read()
                                    .categories
                                    .iter()
                                    .flatten()
                                    .filter(|cat| cat.id != "default")
                                    .cloned()
                                    .map(|category| Category {
                                        server: self.server.clone(),
                                        category: category.into_readable(),
                                    }),
                            ),
                        )
                        .width(Size::Fill),
                ),
            )
    }
}
