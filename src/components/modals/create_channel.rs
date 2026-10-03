use freya::prelude::*;
use stoat_models::v0;

use crate::{
    components::{Dialog, RadioButton, SingleLineEntry, use_modals},
    consume_material_theme, format_error, http,
};

#[derive(PartialEq)]
pub struct CreateChannel {
    pub server: String,
    pub callback: EventHandler<v0::Channel>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
enum ChannelType {
    Text,
    Voice,
}

impl Component for CreateChannel {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let name = use_state(String::new);
        let mut channel_type = use_state(|| ChannelType::Text);
        let max_users = use_state(|| 0);
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(2.).text("Create Channel"))
            .body(
                rect()
                    .spacing(8.)
                    .child(SingleLineEntry::new("Channel Name", name))
                    .child(
                        rect()
                            .spacing(4.)
                            .child(
                                RadioButton::new(channel_type() == ChannelType::Text)
                                    .child("Text Channel")
                                    .on_press(move |_| channel_type.set(ChannelType::Text)),
                            )
                            .child(
                                RadioButton::new(channel_type() == ChannelType::Voice)
                                    .child("Voice Channel")
                                    .on_press(move |_| channel_type.set(ChannelType::Voice)),
                            ),
                    )
                    .maybe_child((channel_type() == ChannelType::Voice).then(|| {
                        let count = max_users();

                        rect()
                            .spacing(8.)
                            .child(label().font_size(14.).text("Max Users"))
                            .child(
                                rect()
                                    .horizontal()
                                    .spacing(8.)
                                    .cross_align(Alignment::Center)
                                    .child(
                                        label()
                                            .width(Size::px(60.))
                                            .font_size(12.)
                                            .color(theme.md.on_surface_variant.as_argb_u32())
                                            .text(format!(
                                                "{} users",
                                                if count == 0 {
                                                    "∞".to_string()
                                                } else {
                                                    count.to_string()
                                                }
                                            )),
                                    )
                                    .child(
                                        Slider::new({
                                            let mut max_users = max_users.clone();
                                            move |value| {
                                                max_users.set(value as usize);
                                            }
                                        })
                                        .scroll_enabled(false)
                                        .value(count as f64)
                                        .step(1.)
                                        .cursor(CursorIcon::Pointer)
                                        .border_fill(Color::TRANSPARENT)
                                        .background(
                                            theme.md.surface_container_highest.as_argb_u32(),
                                        )
                                        .thumb_background(theme.md.primary.as_argb_u32())
                                        .thumb_inner_background(theme.md.primary.as_argb_u32()),
                                    ),
                            )
                    }))
                    .maybe_child(
                        error
                            .read()
                            .clone()
                            .map(|error| label().text(error).color(theme.md.error.as_argb_u32())),
                    ),
            )
            .default_action("Close")
            .action("Create", {
                let server = self.server.clone();
                let callback = self.callback.clone();

                move || {
                    spawn({
                        let name = name.read().clone();
                        let channel_type = channel_type();
                        let server = server.clone();
                        let max_users = max_users();
                        let callback = callback.clone();

                        async move {
                            match http()
                                .create_channel(
                                    &server,
                                    &v0::DataCreateServerChannel {
                                        name,
                                        voice: if channel_type == ChannelType::Voice {
                                            Some(v0::VoiceInformation {
                                                max_users: if max_users == 0 {
                                                    None
                                                } else {
                                                    Some(max_users)
                                                },
                                            })
                                        } else {
                                            None
                                        },
                                        ..Default::default()
                                    },
                                )
                                .await
                            {
                                Ok(response) => {
                                    callback.call(response);
                                    modals.write().pop_modal();
                                }
                                Err(e) => error.set(Some(format_error(&e, "channel"))),
                            };
                        }
                    });

                    false
                }
            })
    }
}
