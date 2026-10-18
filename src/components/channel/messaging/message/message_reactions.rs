use freya::{prelude::*, radio::use_radio};
use indexmap::IndexSet;
use stoat_models::v0;

use crate::{
    AppChannel, SizeExt,
    components::{AnimatedImage, MessageModel, StoatButton, StoatButtonLayoutThemePartialExt},
    consume_material_theme, http,
    types::Tag,
};

#[derive(PartialEq)]
pub struct MessageReactions {
    pub message: MessageModel,
    pub channel: Readable<v0::Channel>,
}

impl Component for MessageReactions {
    fn render(&self) -> impl IntoElement {
        rect()
            .spacing(4.)
            .horizontal()
            .content(Content::wrap_spacing(4.))
            .children(
                self.message
                    .message
                    .reactions
                    .iter()
                    .map(|(emoji, users)| MessageReaction {
                        emoji: emoji.clone(),
                        users: users.clone(),
                        channel: self.channel.clone(),
                        message: self.message.clone(),
                    }),
            )
    }
}

#[derive(PartialEq)]
pub struct MessageReaction {
    pub emoji: String,
    pub users: IndexSet<String>,
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
}

impl Component for MessageReaction {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let radio = use_radio(AppChannel::UserId);
        let user_id = radio.slice_current(|state| state.user_id.as_ref().unwrap());

        let reacted = self.users.contains(&*user_id.read());

        StoatButton::new()
            .corner_radius(12.)
            .on_press({
                let channel = self.channel.clone();
                let message = self.message.clone();
                let emoji = self.emoji.clone();

                move |_| {
                    let emoji = emoji.clone();
                    let channel_id = channel.read().id().to_string();
                    let message_id = message.message.id.clone();

                    spawn(async move {
                        if reacted {
                            let _ = http()
                                .unreact_message(
                                    &channel_id,
                                    &message_id,
                                    &emoji,
                                    &v0::OptionsUnreact {
                                        user_id: None,
                                        remove_all: None,
                                    },
                                )
                                .await;
                        } else {
                            let _ = http().react_message(&channel_id, &message_id, &emoji).await;
                        }
                    });
                }
            })
            .child(
                rect()
                    .key(&self.emoji)
                    .horizontal()
                    .padding(8.)
                    .spacing(8.)
                    .background(theme.md.surface_container_low)
                    .color(theme.md.on_surface)
                    .maybe(reacted, |this| {
                        this.background(theme.md.secondary_container)
                            .color(theme.md.on_secondary_container)
                    })
                    .child(rect().margin((0., 0.7, 0., 1.4)).child(
                        Lazy::new().size(Size::px(16.8)).child({
                            if self.emoji.len() == 26 {
                                let url = format!(
                                    "{}/{}/{}",
                                    http().api_config.features.autumn.url,
                                    Tag::Emojis,
                                    &self.emoji
                                );

                                AnimatedImage::new(url.parse::<Url>().unwrap())
                                    .key(&url)
                                    .sampling_mode(SamplingMode::Trilinear)
                                    .size(Size::px(16.8))
                                    .into_element()
                            } else {
                                let codes = self
                                    .emoji
                                    .chars()
                                    .map(|c| format!("{:x}", c as i32))
                                    .collect::<Vec<String>>()
                                    .join("-");

                                let url = format!(
                                    "https://static.stoat.chat/emoji/fluent-3d/{codes}.svg?v=1"
                                );

                                SvgViewer::new(url.parse::<Url>().unwrap())
                                    .async_rasterization(true)
                                    .size(Size::px(16.8))
                                    .error_renderer(|_| rect().size(Size::px(16.8)).into_element())
                                    .into_element()
                            }
                        }),
                    ))
                    .child(
                        label()
                            .text(self.users.len().to_string())
                            .font_size(14)
                            .line_height(1.5)
                            .max_lines(1),
                    ),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.emoji).into()
    }
}
