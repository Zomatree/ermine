use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Avatar, Dialog, Dropdown, SingleLineEntry},
    http,
};

#[derive(PartialEq)]
pub struct BanMember {
    pub user: String,
    pub server: String,
}

static MESSAGE_DELETION_LENGTHS: &[(Option<i64>, &'static str)] = &[
    (None, "Don't delete messages"),
    (Some(60 * 60), "1 hour"),
    (Some(60 * 60 * 6), "6 hours"),
    (Some(60 * 60 * 24), "1 day"),
    (Some(60 * 60 * 24 * 3), "3 days"),
    (Some(60 * 60 * 24 * 7), "7 days"),
];

impl Component for BanMember {
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

        let reason = use_state(String::new);

        let delete_message_history = use_state(|| None);

        Dialog::new()
            .title(label().line_height(1.5).text("Ban Member"))
            .body(
                rect()
                    .cross_align(Alignment::Center)
                    .spacing(8.)
                    .child(Avatar::new(
                        user.clone().into_readable(),
                        Some(member.into_readable()),
                        64.,
                    ))
                    .child(label().font_size(14.).text(format!(
                        "You are about to ban {}",
                        user.read().username.clone()
                    )))
                    .child(SingleLineEntry::new("Reason", reason))
                    .child(Dropdown::new(
                        "Delete Message History",
                        delete_message_history.into_writable(),
                        MESSAGE_DELETION_LENGTHS
                            .iter()
                            .map(|(length, _)| *length)
                            .collect(),
                        |length| {
                            MESSAGE_DELETION_LENGTHS
                                .iter()
                                .find(|(l, _)| l == length)
                                .unwrap()
                                .1
                                .into_element()
                        },
                    )),
            )
            .default_action("Cancel")
            .action("Ban", {
                let user = self.user.clone();
                let server = self.server.clone();

                move || {
                    let user = user.clone();
                    let server = server.clone();

                    spawn(async move {
                        let reason = reason.read().cloned();
                        let delete_message_seconds = delete_message_history();

                        http()
                            .ban_member(
                                &server,
                                &user,
                                &v0::DataBanCreate {
                                    reason: if !reason.is_empty() {
                                        Some(reason)
                                    } else {
                                        None
                                    },
                                    delete_message_seconds,
                                },
                            )
                            .await
                            .unwrap();
                        {}
                    });

                    true
                }
            })
    }
}
