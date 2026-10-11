use std::time::SystemTime;

use freya::radio::Readable;

use crate::NotificationsSettings;

pub fn is_server_muted(server_id: &str, settings: &Readable<Option<NotificationsSettings>>) -> bool {
    let mute = settings
        .read()
        .as_ref()
        .and_then(|n| n.server_mutes.get(server_id).cloned());

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