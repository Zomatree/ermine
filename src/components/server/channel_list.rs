use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Category, ChannelButton},
    map_readable,
};

#[derive(PartialEq)]
pub struct ChannelList {
    pub server: Readable<v0::Server>,
}

impl Component for ChannelList {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Channels);

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

        rect().padding((0., 0., 8., 8.)).child(
            ScrollView::new().child(
                rect()
                    .cross_align(Alignment::Center)
                    .color(0xff90909a)
                    .child(
                        rect().padding((4., 0.)).children(
                            non_category_channels
                                .read()
                                .iter()
                                .cloned()
                                .filter(|channel_id| radio.read().channels.contains_key(channel_id))
                                .map(|channel_id: String| {
                                    let channel = radio.slice_current(move |state| {
                                        state.channels.get(&channel_id).unwrap()
                                    });

                                    ChannelButton {
                                        channel: channel.into_readable(),
                                    }
                                    .into_element()
                                }),
                        ),
                    )
                    .child(
                        rect().padding((4., 0.)).spacing(8.).children(
                            self.server
                                .read()
                                .categories
                                .iter()
                                .flatten()
                                .filter(|cat| !cat.channels.is_empty() && cat.id != "default")
                                .map(|cat| {
                                    let server = self.server.clone();

                                    let category = map_readable(server, {
                                        let id = cat.id.clone();

                                        move |server| {
                                            server
                                                .categories
                                                .as_ref()
                                                .unwrap()
                                                .iter()
                                                .find(|c| c.id == id)
                                                .unwrap()
                                        }
                                    });

                                    // let channels = cat
                                    //     .channels
                                    //     .into_iter()
                                    //     .filter(|channel_id| radio.read().channels.contains_key(channel_id))
                                    //     .map(|channel_id: String| {
                                    //         radio
                                    //             .slice_current(move |state| {
                                    //                 state.channels.get(&channel_id).unwrap()
                                    //             })
                                    //             .into_readable()
                                    //     })
                                    //     .collect::<Vec<Readable<v0::Channel>>>();

                                    // println!("chanlist: {:?}", channels.iter().map(|c| c.peek().name().map(|s| s.to_string())).collect::<Vec<_>>());

                                    category
                                })
                                .map(|category| {
                                    rect()
                                        .key(category.peek().id.clone())
                                        .child(Category {
                                            server: self.server.clone(),
                                            category: category,
                                        })
                                        .into_element()
                                }),
                        ),
                    )
                    .width(Size::Fill),
            ),
        )
    }
}
