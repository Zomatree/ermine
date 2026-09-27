use freya::{prelude::*, radio::use_radio};
use iso8601_timestamp::{Duration, Timestamp};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Avatar, Dialog, StoatSegmentedButton},
    http,
};

#[derive(PartialEq)]
pub struct TimeoutMember {
    pub user: String,
    pub server: String,
}

static TIMEOUT_LENGTHS: &[(i64, &'static str)] = &[
    (1, "1 min"),
    (5, "5 mins"),
    (10, "10 mins"),
    (60, "1 hour"),
    (60 * 24, "1 day"),
    (60 * 24 * 7, "1 week"),
];

impl Component for TimeoutMember {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Servers);

        let member = radio.slice(AppChannel::Members, {
            let user = self.user.clone();
            let server = self.server.clone();
            move |state| state.members.get(&server).unwrap().get(&user).unwrap()
        });

        let user = radio.slice(AppChannel::Users, {
            let user = self.user.clone();
            move |state| state.users.get(&user).unwrap()
        });

        let duration = use_state(|| 1);

        Dialog::new()
            .title(label().line_height(1.5).text("Timeout Member"))
            .body(
                rect()
                    .width(Size::Fill)
                    .cross_align(Alignment::Center)
                    .spacing(8.)
                    .child(Avatar::new(
                        user.clone().into_readable(),
                        Some(member.into_readable()),
                        64.,
                    ))
                    .child(label().font_size(14.).text(format!(
                        "{} will not be able to interact with this server until the timeout expired.",
                        user.read().username.clone()
                    )))
                    .child(rect().spacing(6.).child(label().font_size(12.).text("Duration")).child(
                        StoatSegmentedButton::new(
                            duration,
                            TIMEOUT_LENGTHS.iter().map(|(length, _)| *length).collect(),
                            |length| {
                                TIMEOUT_LENGTHS
                                    .iter()
                                    .find(|(l, _)| l == length)
                                    .unwrap()
                                    .1
                                    .into_element()
                            },
                        )
                        .height(40.))
                    )
            )
            .default_action("Cancel")
            .action("Timeout", {
                let user = self.user.clone();
                let server = self.server.clone();

                move || {
                    let user = user.clone();
                    let server = server.clone();

                    let end_timestamp = Timestamp::now_utc() + Duration::minutes(duration());

                    spawn(async move { if http().edit_member(&server, &user, &v0::DataMemberEdit {
                        nickname: None,
                        pronouns: None,
                        avatar: None,
                        roles: None,
                        timeout: Some(end_timestamp),
                        can_publish: None,
                        can_receive: None,
                        voice_channel: None,
                        remove: Vec::new(),
                    }).await.is_ok() {} });

                    true
                }
            })
    }
}
