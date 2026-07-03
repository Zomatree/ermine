use freya::{icons::lucide::file_text, prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Avatar, MessageModel},
    http,
    types::Tag,
};

#[derive(PartialEq)]
pub struct MessageReply {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
    pub id: String,
}

impl Component for MessageReply {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Users);
        let users = radio.slice_mut_current(|state| &mut state.users);

        let message_cache = radio.slice_mut(AppChannel::ChannelMessageCache, {
            let channel_id = self.channel.peek().id().to_string();

            move |state| {
                state
                    .channel_message_cache
                    .entry(channel_id.clone())
                    .or_default()
            }
        });

        let reply = use_memo({
            let id = self.id.clone();
            let message_cache = message_cache.clone();

            move || message_cache.read().get(&id).cloned()
        });

        use_hook(|| {
            let exists = reply.read().is_some();

            if !exists {
                let channel = self.channel.clone();
                let id = self.id.clone();
                let mut message_cache = message_cache.clone();

                spawn(async move {
                    let channel_id = channel.peek().id().to_string();

                    if let Ok(message) = http().fetch_message(&channel_id, &id).await {
                        message_cache.write().insert(message.id.clone(), message);
                    }
                });
            }
        });

        let user = use_memo({
            let reply = reply.clone();
            let users = users.clone();

            move || {
                let reply = reply.read();
                let user_id = reply
                    .as_ref()
                    .map(|m| m.author.as_str())
                    .unwrap_or("00000000000000000000000000");

                let users = users.read();

                if let Some(user) = users.get(user_id).cloned() {
                    user
                } else {
                    users.get("00000000000000000000000000").unwrap().clone()
                }
            }
        });

        use_hook(|| {
            let reply = reply.clone();
            let users = users.clone();

            Effect::create_sync(move || {
                let mut users = users.clone();

                spawn(async move {
                    if let Some(reply) = &*reply.read() {
                        if let Some(user) = reply.user.clone() {
                            let mut users = users.write();
                            if !users.contains_key(&user.id) {
                                users.insert(user.id.clone(), user);
                            }
                        } else {
                            let exists = users.read().contains_key(&reply.author);

                            if !exists {
                                let user = if let Some(webhook) = &reply.webhook {
                                    v0::User {
                                        id: reply.author.clone(),
                                        username: webhook.name.clone(),
                                        discriminator: "0000".to_string(),
                                        display_name: None,
                                        avatar: webhook.avatar.clone().map(|id| v0::File {
                                            id,
                                            tag: Tag::Avatars.into(),
                                            filename: String::new(),
                                            metadata: v0::Metadata::File,
                                            content_type: String::new(),
                                            size: 0,
                                            deleted: None,
                                            reported: None,
                                            message_id: None,
                                            user_id: None,
                                            server_id: None,
                                            object_id: None,
                                        }),
                                        relations: Vec::new(),
                                        badges: 0,
                                        status: None,
                                        flags: 0,
                                        privileged: false,
                                        bot: None,
                                        relationship: v0::RelationshipStatus::None,
                                        online: false,
                                    }
                                } else if let Ok(user) = http().fetch_user(&reply.author).await {
                                    user
                                } else {
                                    v0::User {
                                        id: reply.author.clone(),
                                        username: format!("Unknown User {}", reply.author),
                                        discriminator: "0000".to_string(),
                                        display_name: None,
                                        avatar: None,
                                        relations: Vec::new(),
                                        badges: 0,
                                        status: None,
                                        flags: 0,
                                        privileged: false,
                                        bot: None,
                                        relationship: v0::RelationshipStatus::None,
                                        online: false,
                                    }
                                };

                                users.write().insert(user.id.clone(), user);
                            };
                        };
                    };
                });
            });
        });

        rect()
            .height(Size::px(22.))
            .horizontal()
            .font_size(14)
            .spacing(4.)
            .cross_align(Alignment::End)
            .child(
                rect()
                    .height(Size::px(12.))
                    .width(Size::px(22.))
                    .margin((0., 6., 0., 30.))
                    .corner_radius(CornerRadius {
                        top_left: 4.,
                        top_right: 0.,
                        bottom_right: 0.,
                        bottom_left: 0.,
                        smoothing: 0.,
                    })
                    .border(Border::new().fill(0xff45464f).width(BorderWidth {
                        top: 2.,
                        right: 0.,
                        bottom: 0.,
                        left: 2.,
                    })),
            )
            .child(match &*reply.read() {
                Some(reply) => {
                    let has_attachments = reply
                        .attachments
                        .as_ref()
                        .is_some_and(|files| !files.is_empty());

                    rect()
                        .height(Size::px(22.))
                        .horizontal()
                        .spacing(4.)
                        .cross_align(Alignment::Center)
                        .child(
                            rect()
                                .height(Size::px(22.))
                                .horizontal()
                                .spacing(4.)
                                .cross_align(Alignment::Center)
                                .child(Avatar::new(user.clone().into_readable(), None, 14.))
                                .child(label().line_height(1.25).text({
                                    let user = user.read();

                                    if self
                                        .message
                                        .message
                                        .mentions
                                        .as_ref()
                                        .is_some_and(|mentions| mentions.contains(&user.id))
                                    {
                                        format!("@{}", user.username)
                                    } else {
                                        user.username.clone()
                                    }
                                })),
                        )
                        .child(
                            rect()
                                .height(Size::px(22.))
                                .horizontal()
                                .spacing(8.)
                                .cross_align(Alignment::Center)
                                .maybe_child(has_attachments.then(|| {
                                    rect()
                                        .height(Size::px(22.))
                                        .horizontal()
                                        .spacing(4.)
                                        .cross_align(Alignment::Center)
                                        .child(
                                            SvgViewer::new(file_text())
                                                .width(Size::px(16.))
                                                .height(Size::px(16.)),
                                        )
                                        .child(
                                            label()
                                                .font_size(14.)
                                                .text("Sent an attachment")
                                                .font_slant(FontSlant::Italic),
                                        )
                                }))
                                .child(
                                    label()
                                        .text(reply.content.clone().unwrap_or_default())
                                        .max_lines(1)
                                        .text_overflow(TextOverflow::Ellipsis)
                                        .into_element(),
                                ),
                        )
                }
                None => rect()
                    .font_weight(FontWeight::LIGHT)
                    .child("Unknown Message"),
            })
    }
}
