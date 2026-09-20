use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel,
    components::{Avatar, StoatButton},
};

#[derive(PartialEq)]
pub struct FriendButton {
    pub user: Readable<v0::User>,
}

impl Component for FriendButton {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(AppChannel::UserProfile);

        StoatButton::new()
            .on_press({
                let user = self.user.clone();

                move |_| {
                    let id = user.read().id.clone();
                    radio.write().user_profile = Some((id, None));
                }
            })
            .child(
                rect()
                    .horizontal()
                    .padding((8., 16.))
                    .width(Size::Fill)
                    .cross_align(Alignment::Center)
                    .spacing(16.)
                    .child(Avatar::new(self.user.clone(), None, 36.).presence(true))
                    .child(self.user.read().username.clone()),
            )
    }
}
