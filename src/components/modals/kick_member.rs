use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel,
    components::{Avatar, Dialog},
    http,
};

#[derive(PartialEq)]
pub struct KickMember {
    pub user: String,
    pub server: String,
}

impl Component for KickMember {
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
            .title(label().line_height(1.5).text("Kick Member"))
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
                        "You are about to kick {}",
                        user.read().username.clone()
                    ))),
            )
            .default_action("Cancel")
            .action("Kick", {
                let user = self.user.clone();
                let server = self.server.clone();

                move || {
                    let user = user.clone();
                    let server = server.clone();

                    spawn(async move { if http().kick_member(&server, &user).await.is_ok() {} });

                    true
                }
            })
    }
}
