use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Message, MessageModel},
    http,
};

#[derive(PartialEq)]
pub struct MessagePinned {
    pub channel: Readable<v0::Channel>,
}

impl Component for MessagePinned {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Users);
        let users_state = radio.slice_current(|state| &state.users);
        let mut results = use_state(|| None);

        use_hook(move || {
            let channel_id = self.channel.read().id().to_string();

            spawn(async move {
                if let Ok(v0::BulkMessageResponse::MessagesAndUsers {
                    messages,
                    users,
                    members,
                }) = http()
                    .search_channel(
                        &channel_id,
                        &v0::DataMessageSearch {
                            query: None,
                            pinned: Some(true),
                            limit: None,
                            before: None,
                            after: None,
                            sort: v0::MessageSort::Latest,
                            include_users: Some(true),
                        },
                    )
                    .await
                {
                    let mut models = Vec::new();

                    for message in messages {
                        let user = users
                            .iter()
                            .find(|u| u.id == message.author)
                            .cloned()
                            .or_else(|| {
                                users_state
                                    .read()
                                    .get("00000000000000000000000000")
                                    .cloned()
                            })
                            .unwrap()
                            .into_readable();

                        let member = members
                            .as_ref()
                            .and_then(|members| {
                                members.iter().find(|m| m.id.user == message.author)
                            })
                            .cloned()
                            .map(|m| m.into_readable());

                        models.push(MessageModel {
                            message,
                            user,
                            member,
                        });
                    }

                    results.set(Some(models));
                }
            });
        });

        rect()
            .child(rect().padding(8.).child(label().text("Pinned Messages")))
            .child(
                ScrollView::new().child(match results.read().cloned() {
                    None => rect()
                        .width(Size::Fill)
                        .center()
                        .child(CircularLoader::new())
                        .into_element(),

                    Some(messages) => rect()
                        .spacing(8.)
                        .children(messages.into_iter().map(|message| {
                            Message {
                                channel: self.channel.clone(),
                                message,
                            }
                            .into_element()
                        }))
                        .into_element(),
                }),
            )
    }
}
