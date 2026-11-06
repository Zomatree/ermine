use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel, DataChangeUsername, components::{Dialog, SingleLineEntry, use_modals}, consume_material_theme, format_error, http,
};

#[derive(PartialEq)]
pub struct ChangeUsername {}

impl Component for ChangeUsername {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let mut modals = use_modals();

        let radio = use_radio(AppChannel::Users);

        let user =
            radio.slice_current(|state| state.users.get(state.user_id.as_ref().unwrap()).unwrap());

        let username = use_state(|| user.read().username.clone());
        let password = use_state(String::new);
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(2.).text("Change username"))
            .body(
                rect()
                    .spacing(8.)
                    .child(SingleLineEntry::new("Username", username))
                    .child(
                        SingleLineEntry::new("Current Password", password)
                            .placeholder("Enter your current password")
                            .password(),
                    )
                    .maybe_child(
                        error
                            .read()
                            .cloned()
                            .map(|error| label().color(theme.md.error.as_argb_u32()).text(error)),
                    ),
            )
            .default_action("Close")
            .action_with_state(
                "Change",
                !username.read().is_empty() && !password.read().is_empty(),
                {
                    move || {
                        let username = username.read().cloned();
                        let password = password.read().cloned();

                        spawn(async move {
                            match http()
                                .change_username(&DataChangeUsername { username, password })
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
                },
            )
    }
}
