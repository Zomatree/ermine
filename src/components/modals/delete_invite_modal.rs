use freya::prelude::*;

use crate::{
    components::{Dialog, use_modals},
    http,
};

#[derive(PartialEq)]
pub struct DeleteInviteModal {
    pub invite: String,
}

impl Component for DeleteInviteModal {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        Dialog::new()
            .title(label().line_height(1.5).text("Delete message"))
            .body("Are you sure you want to delete this?")
            .default_action("Cancel")
            .action("Delete", {
                let invite = self.invite.clone();

                move || {
                    spawn({
                        let invite = invite.clone();
                        async move {
                            http().delete_invite(&invite).await.unwrap();
                            modals.write().pop_modal();
                        }
                    });

                    false
                }
            })
    }
}
