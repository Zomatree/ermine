use freya::{
    icons::lucide::{at_sign, circle_x, file_text},
    prelude::*,
};
use stoat_models::v0;

use crate::{
    components::{Avatar, ReplyController, ReplyIntent, StoatButton},
    use_material_theme,
};

#[derive(PartialEq)]
pub struct MessageReplyPreview {
    pub replies: ReplyController,
    pub reply: Readable<ReplyIntent>,
    pub channel: Readable<v0::Channel>,
}

impl Component for MessageReplyPreview {
    fn render(&self) -> impl IntoElement {
        let message = self.reply.read().message.clone();
        let theme = use_material_theme();

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
            .child(label().font_size(12).text("Replying to"))
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
                    .child({
                        let user = message.user.read();

                        user.display_name.clone().unwrap_or(user.username.clone())
                    })
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
                                        SvgViewer::new(file_text()).width(Size::px(16.)).height(Size::px(16.)),
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
                                    .child(
                                        SvgViewer::new(at_sign()).width(Size::px(16.)).height(Size::px(16.)),
                                    )
                                    .child(if mention { "ON" } else { "OFF" }),
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
                                SvgViewer::new(circle_x())
                                    .color(theme.md.on_primary_container.as_argb_u32())
                                    .width(Size::px(16.))
                                    .height(Size::px(16.)),
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
