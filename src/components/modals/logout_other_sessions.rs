use freya::prelude::*;
use stoat_models::v0;

use crate::{
    components::{Dialog, ModalValue, use_modals},
    http,
};

#[derive(PartialEq)]
pub struct LogoutOtherSessions {
    pub callback: EventHandler<()>,
}

impl Component for LogoutOtherSessions {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        Dialog::new()
            .title(
                label()
                    .line_height(1.5)
                    .text("Are you sure you want to clear your sessions?"),
            )
            .body(
                label()
                    .line_height(1.5)
                    .text("You cannot undo this action."),
            )
            .default_action("Cancel")
            .action("Accept", {
                let callback = self.callback.clone();

                move || {
                    modals.write().push_modal(ModalValue::MFA {
                        callback: EventHandler::new({
                            let callback = callback.clone();

                            move |ticket: v0::MFATicket| {
                                let callback = callback.clone();

                                spawn_forever(async move {
                                    if http()
                                        .revoke_all_sessions(ticket.token, false)
                                        .await
                                        .is_ok()
                                    {
                                        callback.call(());
                                    };
                                });
                            }
                        }),
                    });

                    false
                }
            })
    }
}
