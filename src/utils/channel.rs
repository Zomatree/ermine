use std::time::SystemTime;

use freya::{
    prelude::*,
    radio::{Radio, Readable},
};
use stoat_models::v0;

use crate::{AppChannel, AppState, ChannelUnread, NotificationBadge, NotificationsSettings};

pub fn is_channel_muted(
    channel_id: &str,
    settings: &Readable<Option<NotificationsSettings>>,
) -> bool {
    let mute = settings
        .read()
        .as_ref()
        .and_then(|n| n.channel_mutes.get(channel_id).cloned());

    mute.is_some_and(|mute| {
        mute.until.is_none_or(|ts| {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                < ts
        })
    })
}

pub fn get_unread_badge(
    channel: &v0::Channel,
    unread: &ChannelUnread,
) -> Option<NotificationBadge> {
    if !unread.mentions.is_empty() {
        Some(NotificationBadge::Mentions(unread.mentions.len()))
    } else {
        let last_message_id = match &channel {
            v0::Channel::TextChannel {
                last_message_id, ..
            }
            | v0::Channel::Group {
                last_message_id, ..
            }
            | v0::Channel::DirectMessage {
                last_message_id, ..
            } => last_message_id.as_ref(),
            _ => None,
        };

        if (unread.last_id.is_none() && last_message_id.is_some())
            || unread
                .last_id
                .as_ref()
                .zip(last_message_id)
                .is_some_and(|(last_id, last_message_id)| last_id < last_message_id)
        {
            Some(NotificationBadge::Unread)
        } else {
            None
        }
    }
}

pub fn get_channel_server(channel: &v0::Channel) -> Option<&str> {
    match channel {
        v0::Channel::TextChannel { server, .. } => Some(server),
        _ => None,
    }
}

pub fn get_channel_name(radio: &Radio<AppState, AppChannel>, channel: &v0::Channel) -> String {
    match channel {
        v0::Channel::SavedMessages { .. } => "Saved Messages".to_string(),
        v0::Channel::DirectMessage { recipients, .. } => {
            let state = radio.peek_state();
            let user_id = state.user_id.as_ref().unwrap();

            let other = recipients.iter().find(|id| id != &user_id).unwrap();

            let users = radio.slice(AppChannel::Users, |state| &state.users);

            users.read().get(other).unwrap().username.clone()
        }
        v0::Channel::Group { name, .. } => name.clone(),
        v0::Channel::TextChannel { name, .. } => name.clone(),
    }
}
