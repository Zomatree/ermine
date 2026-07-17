use freya::{prelude::*, radio::use_radio};

use crate::AppChannel;

#[derive(PartialEq)]
pub struct UserContextMenu {
    pub user_id: String,
    pub server_id: Option<String>,
}

impl Component for UserContextMenu {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Users);

        // let user = radio.slice_current({
        //     let user_id = self.user_id.clone();
        //     move |state| state.users.get(&user_id).unwrap()
        // });

        let mut user_profile =
            radio.slice_mut(AppChannel::UserProfile, |state| &mut state.user_profile);

        rect()
            .content(Content::Fit)
            .child(
                MenuButton::new()
                    .child(label().font_size(14.).text("Open Profile"))
                    .on_press({
                        let user_id = self.user_id.clone();

                        move |_| {
                            *user_profile.write() = Some(user_id.clone());
                        }
                    }),
            )
            .child(
                MenuButton::new()
                    .child(label().font_size(14.).text("Copy User ID"))
                    .on_press({
                        let user_id = self.user_id.clone();

                        move |_| {
                            Clipboard::set(user_id.clone()).unwrap();
                        }
                    }),
            )
    }
}
