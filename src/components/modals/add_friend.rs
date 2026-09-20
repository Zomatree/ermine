use freya::prelude::*;
use stoat_models::v0;

use crate::{
    components::{Dialog, SingleLineEntry, use_modals},
    consume_material_theme, format_error, http,
};

#[derive(PartialEq)]
pub struct AddFriend {}

impl Component for AddFriend {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let mut modals = use_modals();

        let username = use_state(String::new);
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(2.).text("Add a new friend"))
            .body(
                rect()
                    .child(SingleLineEntry::new("Username", username).placeholder("username#1234"))
                    .maybe_child(
                        error
                            .read()
                            .cloned()
                            .map(|error| label().color(theme.md.error.as_argb_u32()).text(error)),
                    ),
            )
            .default_action("Close")
            .action_with_state("Send Request", {
                let username = username.read();

                !username.is_empty() && username.split_once('#').is_some()
            }, {
                move || {
                    let username = username.read().cloned();

                    spawn(async move {
                        match http()
                            .send_friend_request(&v0::DataSendFriendRequest { username })
                            .await
                        {
                            Ok(_) => {
                                modals.write().pop_modal();
                            }
                            Err(e) => {
                                error.set(Some(format_error(&e, "user")));
                            }
                        }
                    });

                    false
                }
            })
    }
}
