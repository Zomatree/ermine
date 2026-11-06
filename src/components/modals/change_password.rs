use freya::prelude::*;
use stoat_models::v0;

use crate::{
    components::{Dialog, ModalValue, SingleLineEntry, use_modals},
    consume_material_theme, format_error, http,
};

#[derive(PartialEq)]
pub struct ChangePassword {}

impl Component for ChangePassword {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let mut modals = use_modals();

        let new_password = use_state(String::new);
        let current_password = use_state(String::new);
        let mut error = use_state(|| None);

        let change_password = {
            move |token| async move {
                match http()
                    .change_password(
                        token,
                        &v0::DataChangePassword {
                            password: new_password.read().cloned(),
                            current_password: current_password.read().cloned(),
                        },
                    )
                    .await
                {
                    Ok(_) => {
                        modals.write().pop_modal();
                    }
                    Err(e) => {
                        error.set(Some(format_error(&e, "user")));
                    }
                }
            }
        };

        Dialog::new()
            .title(label().line_height(2.).text("Change password"))
            .body(
                rect()
                    .spacing(8.)
                    .child(SingleLineEntry::new("New Password", new_password).placeholder("Enter a new password"))
                    .child(
                        SingleLineEntry::new("Current Password", current_password)
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
                !new_password.read().is_empty() && !current_password.read().is_empty(),
                {
                    move || {
                        spawn_in_window(async move {
                            if http().fetch_mfa_status().await.unwrap().totp_mfa {
                                modals.clone().write().push_modal(ModalValue::MFA {
                                    callback: EventHandler::new(move |ticket: v0::MFATicket| {
                                        spawn_in_window(change_password(Some(ticket.token)));
                                    }),
                                });
                            } else {
                                change_password(None).await;
                            }
                        });

                        false
                    }
                },
            )
    }
}
