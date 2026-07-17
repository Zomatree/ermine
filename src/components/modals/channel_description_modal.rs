use std::borrow::Cow;

use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Dialog, MarkdownViewer},
};

#[derive(PartialEq)]
pub struct ChannelDescriptionModal {
    pub channel: String,
}

impl Component for ChannelDescriptionModal {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Channels);
        let channels = radio.read();

        let channel = channels.channels.get(&self.channel).unwrap();

        let server = use_hook(|| {
            if let v0::Channel::TextChannel { server, .. } = channel {
                let server = server.clone();

                Some(
                    radio
                        .slice(AppChannel::Servers, move |state| {
                            state.servers.get(&server).unwrap()
                        })
                        .into_readable(),
                )
            } else {
                None
            }
        });

        let channel_name = match &channel {
            v0::Channel::DirectMessage { recipients, .. } => {
                let user_id = radio.peek_state().user_id.clone().unwrap();

                let other = recipients
                    .iter()
                    .find(|&id| id != &*user_id)
                    .unwrap()
                    .clone();

                let user = radio.slice(AppChannel::Users, move |state| {
                    state.users.get(&other).unwrap()
                });

                Cow::Owned(user.read().username.clone())
            }
            v0::Channel::Group { name, .. } | v0::Channel::TextChannel { name, .. } => {
                Cow::Owned(name.clone())
            }
            v0::Channel::SavedMessages { .. } => Cow::Borrowed("Saved Messages"),
        };

        let channel_description = if let v0::Channel::TextChannel { description, .. }
        | v0::Channel::Group { description, .. } = &channel
        {
            description.clone()
        } else {
            None
        };

        Dialog::new()
            .title(label().line_height(2.).text(format!("#{channel_name}")))
            .body(MarkdownViewer::new(
                channel_description.unwrap_or_default(),
                server,
            ))
            .default_action("Close")
    }
}
