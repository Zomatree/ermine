use freya::{prelude::*, radio::use_radio};
use jiff::{Timestamp, tz::TimeZone};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{
        Avatar, MessageContent, MessageModel, MessageReply, SystemMessage, UserCard,
        UserContextMenu, use_floating,
    },
    consume_material_theme, member_display_color,
};

#[derive(PartialEq)]
pub struct Message {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
}

impl Component for Message {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);

        let theme = consume_material_theme();

        let server_id = use_hook({
            move || {
                if let Some(member) = &self.message.member {
                    Some(member.read().id.server.clone())
                } else {
                    None
                }
            }
        });

        let server = use_memo({
            let server_id = server_id.clone();

            move || {
                if let Some(server_id) = &server_id {
                    radio.read().servers.get(server_id).cloned()
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

        if let Some(system) = self.message.message.system.clone() {
            SystemMessage { message: self.message.clone(), system, server_id: server_id.clone() }.into_element()
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
                                        .on_secondary_down({
                                            let user = self.message.user.clone();
                                            let server = server.clone();

                                            move |e: Event<PressEventData>| {
                                                e.stop_propagation();
                                                ContextMenu::open_from_event(
                                                    &e,
                                                    Menu::new().child(UserContextMenu {
                                                        user_id: user.read().id.clone(),
                                                        server_id: server
                                                            .read()
                                                            .as_ref()
                                                            .map(|s| s.id.clone()),
                                                    }),
                                                );
                                            }
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
                .into_element()
        }
    }

    fn render_key(&self) -> DiffKey {
        (&self.message.message.id).into()
    }
}
