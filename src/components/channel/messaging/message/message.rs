use freya::{
    icons::lucide::{
        arrow_left, arrow_right, circle_x, image, info, key, pin, pin_off, plus, shield_x, tag,
        text_align_start, volume_2, x,
    },
    prelude::*,
    radio::use_radio,
};
use jiff::{Timestamp, tz::TimeZone};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Avatar, MessageContent, MessageModel, MessageReply, UserCard, use_floating},
    member_display_color, use_material_theme,
};

#[derive(PartialEq)]
pub struct Message {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
}

impl Component for Message {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);
        let users = radio.slice(AppChannel::Users, |state| &state.users);

        let theme = use_material_theme();

        let server = use_memo({
            let member = self.message.member.clone();

            move || {
                if let Some(member) = &member {
                    let member = member.read();

                    radio.read().servers.get(&member.id.server).cloned()
                } else {
                    None
                }
            }
        });

        let role_color = use_memo({
            let member = self.message.member.clone();

            move || {
                if let Some(member) = &member
                    && let Some(server) = &*server.read()
                {
                    member_display_color(&member.read(), server)
                } else {
                    None
                }
            }
        });

        let display_name = use_memo({
            let user = self.message.user.clone();
            let member = self.message.member.clone();

            move || {
                member
                    .as_ref()
                    .and_then(|member| member.read().nickname.clone())
                    .unwrap_or_else(|| {
                        let user = user.read();

                        user.display_name.as_ref().unwrap_or(&user.username).clone()
                    })
            }
        });

        let floating = use_floating();

        let open_profile = {
            let floating = floating.clone();
            let user = self.message.user.clone();
            let member = self.message.member.clone();

            move || {
                floating.clone().set(Some(
                    UserCard {
                        user: user.clone(),
                        member: member.clone(),
                    }
                    .into_element(),
                ));
            }
        };

        if let Some(system) = &self.message.message.system {
            rect()
                .horizontal()
                .spacing(8.)
                .font_size(14.)
                .child(
                    rect()
                        .width(Size::px(70.))
                        .height(Size::px(20.))
                        .center()
                        .color(theme.md.primary.as_argb_u32())
                        .child(
                            SvgViewer::new(match system {
                                v0::SystemMessage::Text { .. } => info(),
                                v0::SystemMessage::UserAdded { .. } => plus(),
                                v0::SystemMessage::UserRemove { .. } => x(),
                                v0::SystemMessage::UserJoined { .. } => arrow_right(),
                                v0::SystemMessage::UserLeft { .. } => arrow_left(),
                                v0::SystemMessage::UserKicked { .. } => circle_x(),
                                v0::SystemMessage::UserBanned { .. } => shield_x(),
                                v0::SystemMessage::ChannelRenamed { .. } => tag(),
                                v0::SystemMessage::ChannelDescriptionChanged { .. } => {
                                    text_align_start()
                                }
                                v0::SystemMessage::ChannelIconChanged { .. } => image(),
                                v0::SystemMessage::ChannelOwnershipChanged { .. } => key(),
                                v0::SystemMessage::MessagePinned { .. } => pin(),
                                v0::SystemMessage::MessageUnpinned { .. } => pin_off(),
                                v0::SystemMessage::CallStarted { .. } => volume_2(),
                            })
                            .width(Size::px(16.))
                            .height(Size::px(16.)),
                        ),
                )
                .child(match system {
                    v0::SystemMessage::Text { content } => {
                        rect().child(label().text(content.clone()))
                    }
                    v0::SystemMessage::UserAdded { id, by } => {
                        let user = users.read().get(id).unwrap().clone();
                        let by = users.read().get(by).unwrap().clone();

                        rect().child(label().text(format!(
                            "{} has been added by {}",
                            user.username, by.username
                        )))
                    }
                    v0::SystemMessage::UserRemove { id, by } => {
                        let user = users.read().get(id).unwrap().clone();
                        let by = users.read().get(by).unwrap().clone();

                        rect().child(label().text(format!(
                            "{} has been removed by {}",
                            user.username, by.username
                        )))
                    }
                    v0::SystemMessage::UserJoined { id } => {
                        let user = users.read().get(id).unwrap().clone();

                        rect().child(label().text(format!("{} joined the server", user.username)))
                    }
                    v0::SystemMessage::UserLeft { id } => {
                        let user = users.read().get(id).unwrap().clone();

                        rect().child(label().text(format!("{} left the server", user.username)))
                    }
                    v0::SystemMessage::UserKicked { id } => {
                        let user = users.read().get(id).unwrap().clone();

                        rect().child(
                            label()
                                .text(format!("{} has been kicked from the server", user.username)),
                        )
                    }
                    v0::SystemMessage::UserBanned { id } => {
                        let user = users.read().get(id).unwrap().clone();

                        rect().child(
                            label()
                                .text(format!("{} has been banned from the server", user.username)),
                        )
                    }
                    v0::SystemMessage::ChannelRenamed { name, by } => {
                        let user = users.read().get(by).unwrap().clone();

                        rect().child(label().text(format!(
                            "{} updated the group name to {}",
                            user.username, name
                        )))
                    }
                    v0::SystemMessage::ChannelDescriptionChanged { by } => {
                        let user = users.read().get(by).unwrap().clone();

                        rect().child(
                            label()
                                .text(format!("{} updated the group description", user.username)),
                        )
                    }
                    v0::SystemMessage::ChannelIconChanged { by } => {
                        let user = users.read().get(by).unwrap().clone();

                        rect().child(
                            label().text(format!("{} updated the group icon", user.username)),
                        )
                    }
                    v0::SystemMessage::ChannelOwnershipChanged { from, to } => {
                        let from = users.read().get(from).unwrap().clone();
                        let to = users.read().get(to).unwrap().clone();

                        rect().child(label().text(format!(
                            "{} transferred group ownership to {}",
                            from.username, to.username
                        )))
                    }
                    v0::SystemMessage::MessagePinned { id, by } => {
                        let user = users.read().get(by).unwrap().clone();

                        rect().child(label().text(format!("{} pinned", user.username)))
                    }
                    v0::SystemMessage::MessageUnpinned { id, by } => {
                        let user = users.read().get(by).unwrap().clone();

                        rect().child(label().text(format!("{} unpinned", user.username)))
                    }
                    v0::SystemMessage::CallStarted { by, finished_at } => {
                        let user = users.read().get(by).unwrap().clone();

                        rect().child(label().text(format!("{} started a call", user.username)))
                    }
                })
        } else {
            rect()
                .child(
                    rect().children(self.message.message.replies.iter().flatten().cloned().map(
                        |id| {
                            rect()
                                .key(&id)
                                .child(MessageReply {
                                    channel: self.channel.clone(),
                                    message: self.message.clone(),
                                    id,
                                })
                                .into_element()
                        },
                    )),
                )
                .child(
                    rect()
                        .horizontal()
                        .spacing(8.)
                        .child(
                            rect()
                                .horizontal()
                                .main_align(Alignment::End)
                                .width(Size::px(54.))
                                .padding((2., 4.))
                                .child(
                                    rect()
                                        .on_pointer_enter(move |_| {
                                            Cursor::set(CursorIcon::Pointer);
                                        })
                                        .on_pointer_leave(move |_| {
                                            Cursor::set(CursorIcon::default());
                                        })
                                        .on_press({
                                            let open_profile = open_profile.clone();
                                            move |_| open_profile()
                                        })
                                        .child(Avatar::new(
                                            self.message.user.clone(),
                                            self.message.member.clone(),
                                            36.,
                                        )),
                                ),
                        )
                        .child(
                            rect()
                                .spacing(2.)
                                .padding((0., 15., 0., 0.))
                                .child(
                                    rect()
                                        .horizontal()
                                        .spacing(8.)
                                        .cross_align(Alignment::Center)
                                        .font_size(14)
                                        .child(
                                            label()
                                                .text(display_name.read().clone())
                                                .map(
                                                    role_color.read().clone(),
                                                    |mut this, color| {
                                                        this.get_text_style_data().color =
                                                            Some(color);
                                                        this
                                                    },
                                                )
                                                .line_height(1.5)
                                                .on_pointer_enter(move |_| {
                                                    Cursor::set(CursorIcon::Pointer);
                                                })
                                                .on_pointer_leave(move |_| {
                                                    Cursor::set(CursorIcon::default());
                                                })
                                                .on_press({
                                                    let open_profile = open_profile.clone();
                                                    move |_| open_profile()
                                                }),
                                        )
                                        .child(
                                            label()
                                                .text({
                                                    let datetime = Timestamp::try_from(
                                                        ulid::Ulid::from_string(
                                                            &self.message.message.id,
                                                        )
                                                        .unwrap()
                                                        .datetime(),
                                                    )
                                                    .unwrap()
                                                    .to_zoned(TimeZone::system());

                                                    let now = Timestamp::now()
                                                        .to_zoned(TimeZone::system());

                                                    if datetime.date() == now.date() {
                                                        format!(
                                                            "Today at {:02}:{:02}",
                                                            datetime.hour(),
                                                            datetime.minute()
                                                        )
                                                    } else if now.date().yesterday().unwrap()
                                                        == datetime.date()
                                                    {
                                                        format!(
                                                            "Yesterday at {:02}:{:02}",
                                                            datetime.hour(),
                                                            datetime.minute()
                                                        )
                                                    } else {
                                                        format!(
                                                            "{:02}/{:02}/{}",
                                                            datetime.day(),
                                                            datetime.month(),
                                                            datetime.year()
                                                        )
                                                    }
                                                })
                                                .color(theme.md.outline.as_argb_u32())
                                                .font_size(12),
                                        )
                                        .maybe_child(self.message.message.edited.as_ref().map(
                                            |_ts| {
                                                label()
                                                    .text("(edited)")
                                                    .font_size(12)
                                                    .color(theme.md.outline.as_argb_u32())
                                            },
                                        )),
                                )
                                .child(MessageContent {
                                    channel: self.channel.clone(),
                                    message: self.message.clone(),
                                }),
                        ),
                )
        }
    }
}
