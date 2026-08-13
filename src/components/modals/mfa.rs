use freya::prelude::*;
use stoat_models::v0;

use crate::{
    components::{Dialog, SingleLineEntry, use_modals},
    consume_material_theme, format_error, http,
};

#[derive(PartialEq)]
pub struct MFA {
    pub callback: EventHandler<v0::MFATicket>,
}

impl Component for MFA {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let mut mfa = use_state(|| v0::MultiFactorStatus::default());
        let code = use_state(String::new);
        let mut error = use_state(|| None::<String>);

        use_hook(|| {
            spawn(async move {
                if let Ok(status) = http().fetch_mfa_status().await {
                    mfa.set(status);
                };
            });
        });

        let is_totp = mfa.read().totp_mfa;

        Dialog::new()
            .title(label().line_height(1.5).text("Confirm action"))
            .body(
                rect()
                    .child(label().text("Please confirm using a selected method."))
                    .child(
                        SingleLineEntry::new(
                            {
                                if is_totp {
                                    "Authenticator App"
                                } else {
                                    "Password"
                                }
                            },
                            code,
                        )
                        .mode(if is_totp {
                            InputMode::Shown
                        } else {
                            InputMode::Hidden('•')
                        }),
                    )
                    .maybe_child(
                        error
                            .read()
                            .cloned()
                            .map(|error| label().color(theme.md.error.as_argb_u32()).text(error)),
                    ),
            )
            .default_action("Cancel")
            .action("Confirm", {
                let callback = self.callback.clone();
                move || {
                    let code = code.read().cloned();
                    let callback = callback.clone();

                    spawn(async move {
                        match http()
                            .create_mfa_ticket(&if is_totp {
                                v0::MFAResponse::Totp { totp_code: code }
                            } else {
                                v0::MFAResponse::Password { password: code }
                            })
                            .await
                        {
                            Ok(ticket) => {
                                callback.call(ticket);
                                modals.write().pop_modal();
                            }
                            Err(e) => error.set(Some(format_error(&e, "MFA"))),
                        }
                    });

                    false
                }
            })
    }
}
