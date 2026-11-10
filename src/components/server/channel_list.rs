use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, calculate_server_permissions,
    components::{Category, ChannelButton, ChannelListContextMenu, stoat_scrollbar},
    user_permissions_query,
};

#[derive(PartialEq)]
pub struct ChannelList {
    pub server: Readable<v0::Server>,
}

impl Component for ChannelList {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Channels);
        let channels = radio.slice_current(|state| &state.channels);

        let non_category_channels = {
            let server = self.server.read();
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
        };

        rect()
            .padding((0., 0., 8., 8.))
            .on_secondary_down({
                let server = self.server.clone();
                move |_| {
                    let server = server.read().clone();

                    let mut query = user_permissions_query(radio).server(server.clone());

                    let permissions = calculate_server_permissions(&mut query);

                    ContextMenu::open_from_down(Menu::new().key(&server.id).child(
                        ChannelListContextMenu {
                            server_id: server.id.clone(),
                            current_permissions: permissions,
                        },
                    ))
                }
            })
            .child(
                ScrollView::new().scrollbar(stoat_scrollbar).child(
                    rect()
                        .cross_align(Alignment::Center)
                        .color(0xff90909a)
                        .child(
                            rect().padding((0., 0., 4., 0.)).children(
                                non_category_channels
                                    .into_iter()
                                    .filter(|channel_id| {
                                        channels.read().contains_key(channel_id)
                                    })
                                    .map(|channel| {
                                        let channel = radio.slice_current(move |state| state.channels.get(&channel).unwrap());

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
