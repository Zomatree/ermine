use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, SizeExt,
    components::{
        Avatar, ReplyController, ReplyIntent, StoatButton,
        material::{
            MaterialIcon,
            filled::{alternate_email, description},
            outlined::cancel,
        },
    },
    consume_material_theme, member_display_color,
};

#[derive(PartialEq)]
pub struct MessageReplyPreview {
    pub replies: ReplyController,
    pub reply: Readable<ReplyIntent>,
    pub channel: Readable<v0::Channel>,
}

impl Component for MessageReplyPreview {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);

        let message = self.reply.read().message.clone();
        let theme = consume_material_theme();

        let server = use_hook({
            move || {
                if let v0::Channel::TextChannel { server, .. } = &*self.channel.read() {
                    let server = server.clone();

                    Some(radio.slice_current(move |state| state.servers.get(&server).unwrap()))
                } else {
                    None
                }
            }
        });

        let role_color = use_memo({
            let member = message.member.clone();
            let server = server.clone();

            move || {
                if let Some(member) = &member
                    && let Some(server) = &server
                {
                    member_display_color(&*member.read(), &*server.read())
                } else {
                    None
                }
            }
        });

        let display_name = use_memo({
            let member = message.member.clone();
            let user = message.user.clone();

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

        let has_attachments = message
            .message
            .attachments
            .as_ref()
            .is_some_and(|files| !files.is_empty());

        rect()
            .background(theme.md.primary_container.as_argb_u32())
            .color(theme.md.on_primary_container.as_argb_u32())
            .corner_radius(16.)
            .overflow(Overflow::Clip)
            .content(Content::Flex)
            .padding((8., 16., 8., 16.))
            .horizontal()
            .content(Content::Flex)
            .font_size(14)
            .spacing(4.)
            .cross_align(Alignment::Center)
            .child(
                label()
                    .font_size(12)
                    .color(theme.md.on_primary_container.as_argb_u32())
                    .text("Replying to"),
            )
            .child(
                rect()
                    .horizontal()
                    .spacing(4.)
                    .cross_align(Alignment::Center)
                    .width(Size::flex(1.))
                    .child(Avatar::new(
                        message.user.clone(),
                        message.member.clone(),
                        14.,
                    ))
                    .child(
                        label()
                            .map(role_color.read().cloned(), |this, color| this.color(color))
                            .text(display_name.read().cloned()),
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
                                    .child(MaterialIcon::new(description()).size(Size::px(16.)))
                                    .child(
                                        label()
                                            .font_size(14.)
                                            .text("Sent an attachment")
                                            .font_slant(FontSlant::Italic),
                                    )
                            }))
                            .child(
                                label()
                                    .text(message.message.content.clone().unwrap_or_default())
                                    .width(Size::flex(1.))
                                    .max_lines(1)
                                    .text_overflow(TextOverflow::Ellipsis)
                                    .into_element(),
                            ),
                    ),
            )
            .child(
                rect()
                    .horizontal()
                    .spacing(15.)
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::End)
                    .child({
                        let mention = self.reply.read().mention;

                        StoatButton::new()
                            .child(
                                rect()
                                    .spacing(4.)
                                    .horizontal()
                                    .cross_align(Alignment::Center)
                                    .color(
                                        if mention {
                                            theme.md.on_primary_container
                                        } else {
                                            theme.md.outline
                                        }
                                        .as_argb_u32(),
                                    )
                                    .child(MaterialIcon::new(alternate_email()).size(Size::px(16.)))
                                    .child(label().text(if mention { "ON" } else { "OFF" })),
                            )
                            .on_press({
                                let mut replies = self.replies.clone();
                                let message_id = message.message.id.clone();

                                move |_| {
                                    replies.toggle_mention(&message_id);
                                }
                            })
                    })
                    .child(
                        StoatButton::new()
                            .child(
                                MaterialIcon::new(cancel())
                                    .color(theme.md.on_primary_container.as_argb_u32())
                                    .size(Size::px(16.)),
                            )
                            .on_press({
                                let mut replies = self.replies.clone();
                                let message_id = message.message.id.clone();

                                move |_| {
                                    replies.remove_reply(&message_id);
                                }
                            }),
                    ),
            )
    }
}
