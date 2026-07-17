use freya::prelude::*;
use stoat_models::v0;

use crate::{
    SizeExt, components::{
        MaterialIcon, MessageModel, UserMention,
        material::filled::{
            add, arrow_back, arrow_forward, cancel, clear, format_align_left, image, info, key,
            local_offer, local_police, push_pin, volume_up,
        },
    }, consume_material_theme
};

#[derive(PartialEq)]
pub struct SystemMessage {
    pub message: MessageModel,
    pub system: v0::SystemMessage,
    pub server_id: Option<String>,
}

impl Component for SystemMessage {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        rect()
            .horizontal()
            .spacing(8.)
            .font_size(14.)
            .child({
                let (icon, color) = match &self.system {
                    v0::SystemMessage::Text { .. } => (info(), theme.md.primary),
                    v0::SystemMessage::UserAdded { .. } => (add(), theme.md.primary),
                    v0::SystemMessage::UserRemove { .. } => (clear(), theme.md.error),
                    v0::SystemMessage::UserJoined { .. } => (arrow_forward(), theme.md.primary),
                    v0::SystemMessage::UserLeft { .. } => (arrow_back(), theme.md.error),
                    v0::SystemMessage::UserKicked { .. } => (cancel(), theme.md.error),
                    v0::SystemMessage::UserBanned { .. } => (local_police(), theme.md.error),
                    v0::SystemMessage::ChannelRenamed { .. } => (local_offer(), theme.md.primary),
                    v0::SystemMessage::ChannelDescriptionChanged { .. } => {
                        (format_align_left(), theme.md.primary)
                    }
                    v0::SystemMessage::ChannelIconChanged { .. } => (image(), theme.md.primary),
                    v0::SystemMessage::ChannelOwnershipChanged { .. } => (key(), theme.md.primary),
                    v0::SystemMessage::MessagePinned { .. } => (push_pin(), theme.md.primary),
                    v0::SystemMessage::MessageUnpinned { .. } => (push_pin(), theme.md.primary),
                    v0::SystemMessage::CallStarted { .. } => (volume_up(), theme.md.primary),
                };

                rect()
                    .width(Size::px(70.))
                    .height(Size::px(20.))
                    .center()
                    .color(color.as_argb_u32())
                    .child(MaterialIcon::new(icon).size(Size::px(16.)))
            })
            .child(
                match self.system.clone() {
                    v0::SystemMessage::Text { content } => paragraph().span(content),
                    v0::SystemMessage::UserAdded { id, by } => paragraph()
                        .child(UserMention {
                            user_id: id,
                            server_id: self.server_id.clone(),
                        })
                        .span(" has been added by ")
                        .child(UserMention {
                            user_id: by,
                            server_id: self.server_id.clone(),
                        }),
                    v0::SystemMessage::UserRemove { id, by } => paragraph()
                        .child(UserMention {
                            user_id: id,
                            server_id: self.server_id.clone(),
                        })
                        .span(" has been removed by ")
                        .child(UserMention {
                            user_id: by,
                            server_id: self.server_id.clone(),
                        }),
                    v0::SystemMessage::UserJoined { id } => paragraph()
                        .child(UserMention {
                            user_id: id,
                            server_id: self.server_id.clone(),
                        })
                        .span(" joined the server"),
                    v0::SystemMessage::UserLeft { id } => paragraph()
                        .child(UserMention {
                            user_id: id,
                            server_id: self.server_id.clone(),
                        })
                        .span(" left the server"),
                    v0::SystemMessage::UserKicked { id } => paragraph()
                        .child(UserMention {
                            user_id: id,
                            server_id: self.server_id.clone(),
                        })
                        .span(" has been kicked from the server"),
                    v0::SystemMessage::UserBanned { id } => paragraph()
                        .child(UserMention {
                            user_id: id,
                            server_id: self.server_id.clone(),
                        })
                        .span(" has been banned from the server"),
                    v0::SystemMessage::ChannelRenamed { name, by } => paragraph()
                        .child(UserMention {
                            user_id: by,
                            server_id: self.server_id.clone(),
                        })
                        .span(" updated the group name to ")
                        .span(Span::new(name).font_weight(FontWeight::BOLD)),
                    v0::SystemMessage::ChannelDescriptionChanged { by } => paragraph()
                        .child(UserMention {
                            user_id: by,
                            server_id: self.server_id.clone(),
                        })
                        .span(" updated the group description"),
                    v0::SystemMessage::ChannelIconChanged { by } => paragraph()
                        .child(UserMention {
                            user_id: by,
                            server_id: self.server_id.clone(),
                        })
                        .span(" updated the group icon"),
                    v0::SystemMessage::ChannelOwnershipChanged { from, to } => paragraph()
                        .child(UserMention {
                            user_id: from,
                            server_id: self.server_id.clone(),
                        })
                        .span(" transferred group ownership to ")
                        .child(UserMention {
                            user_id: to,
                            server_id: self.server_id.clone(),
                        }),
                    v0::SystemMessage::MessagePinned { id, by } => paragraph()
                        .child(UserMention {
                            user_id: by,
                            server_id: self.server_id.clone(),
                        })
                        .span(" pinned <TODO>"),
                    v0::SystemMessage::MessageUnpinned { id, by } => paragraph()
                        .child(UserMention {
                            user_id: by,
                            server_id: self.server_id.clone(),
                        })
                        .span(" unpinned <TODO>"),
                    v0::SystemMessage::CallStarted { by, finished_at } => {
                        let p = paragraph().child(UserMention {
                            user_id: by,
                            server_id: self.server_id.clone(),
                        });

                        if let Some(_timestamp) = finished_at {
                            p.span(" started a call that lasted ").child("<TODO>")
                        } else {
                            p.span(" started a call")
                        }
                    }
                }
                .line_height(1.5),
            )
    }
}
