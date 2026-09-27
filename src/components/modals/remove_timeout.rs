use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Avatar, Dialog},
    http,
};

#[derive(PartialEq)]
pub struct RemoveTimeoutMember {
    pub user: String,
    pub server: String,
}

impl Component for RemoveTimeoutMember {
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

        Dialog::new()
            .title(label().line_height(1.5).text("Remove Timeout"))
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
                        "{}'s timeout will be removed and they will be able to interact with the server again.",
                        user.read().username.clone()
                    ))),
            )
            .default_action("Cancel")
            .action("Remove Timeout", {
                let user = self.user.clone();
                let server = self.server.clone();

                move || {
                    let user = user.clone();
                    let server = server.clone();

                    spawn(async move { if http().edit_member(&server, &user, &v0::DataMemberEdit {
                        nickname: None,
                        pronouns: None,
                        avatar: None,
                        roles: None,
                        timeout: None,
                        can_publish: None,
                        can_receive: None,
                        voice_channel: None,
                        remove: vec![v0::FieldsMember::Timeout],
                    }).await.is_ok() {} });

                    true
                }
            })
    }
}
