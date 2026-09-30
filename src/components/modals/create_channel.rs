use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, components::{Dialog, SingleLineEntry, StoatSegmentedButton, use_modals}, consume_material_theme, format_error, http
};

#[derive(PartialEq)]
pub struct CreateChannel {
    pub server: String,
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
        let radio = use_radio(AppChannel::SelectedChannel);
        let selected_channel = radio.slice_mut_current(|state| &mut state.selected_channel);

        let name = use_state(String::new);
        let channel_type = use_state(|| ChannelType::Text);
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(2.).text("Create Channel"))
            .body(
                rect()
                    .spacing(8.)
                    .child(SingleLineEntry::new("Channel Name", name))
                    .child(StoatSegmentedButton::new(
                        channel_type,
                        vec![ChannelType::Text, ChannelType::Voice],
                        |ty| {
                            match ty {
                                ChannelType::Text => "Text Channel",
                                ChannelType::Voice => "Voice Channel",
                            }
                            .into_element()
                        },
                    ))
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

                move || {
                    spawn({
                        let name = name.read().clone();
                        let channel_type = channel_type();
                        let server = server.clone();
                        let mut selected_channel = selected_channel.clone();

                        async move {
                            match http()
                                .create_channel(
                                    &server,
                                    &v0::DataCreateServerChannel {
                                        name,
                                        voice: if channel_type == ChannelType::Voice {
                                            Some(v0::VoiceInformation { max_users: None })
                                        } else {
                                            None
                                        },
                                        ..Default::default()
                                    },
                                )
                                .await
                            {
                                Ok(response) => {
                                    selected_channel.set(Some((response.id().to_string(), None)));
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
