use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, components::{Message, MessageModel, StoatSegmentedButton}, http
};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum MessageSort {
    Relevance,
    Latest,
    Oldest,
}

impl Into<v0::MessageSort> for MessageSort {
    fn into(self) -> v0::MessageSort {
        match self {
            MessageSort::Relevance => v0::MessageSort::Relevance,
            MessageSort::Latest => v0::MessageSort::Latest,
            MessageSort::Oldest => v0::MessageSort::Oldest,
        }
    }
}

#[derive(PartialEq)]
pub struct MessageSearch {
    pub query: String,
    pub channel: Readable<v0::Channel>,
}

impl Component for MessageSearch {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Users);
        let users_state = radio.slice_current(|state| &state.users);

        let sort = use_state(|| MessageSort::Latest);
        let mut results = use_state(|| None);
        let mut task = use_state(|| None::<TaskHandle>);

        let query = use_reactive(&self.query);

        use_hook(move || {
            let channel_id = self.channel.read().id().to_string();

            Effect::create(move || {
                let query = query.read().clone();
                let sort = sort.read().cloned();
                let channel_id = channel_id.clone();
                let users_state = users_state.clone();

                results.set(None);
                let mut task = task.write();
                task.as_ref().map(|t| t.cancel());

                *task = Some(spawn(async move {
                    if let Ok(v0::BulkMessageResponse::MessagesAndUsers {
                        messages,
                        users,
                        members,
                    }) = http()
                        .search_channel(
                            &channel_id,
                            &v0::DataMessageSearch {
                                query: Some(query),
                                pinned: None,
                                limit: None,
                                before: None,
                                after: None,
                                sort: sort.into(),
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
                                .or_else(|| users_state.read().get("00000000000000000000000000").cloned())
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
                }));
            })
        });

        rect()
            .spacing(8.)
            .child(
                StoatSegmentedButton::new(
                    sort,
                    vec![
                        MessageSort::Relevance,
                        MessageSort::Latest,
                        MessageSort::Oldest,
                    ],
                    |sort| {
                        match sort {
                            MessageSort::Relevance => "Relevance",
                            MessageSort::Latest => "Latest",
                            MessageSort::Oldest => "Oldest",
                        }
                        .into_element()
                    },
                )
                .height(40.),
            )
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
