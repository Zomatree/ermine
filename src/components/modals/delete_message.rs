use freya::prelude::*;

use crate::{
    components::{Dialog, use_modals},
    http,
};

#[derive(PartialEq)]
pub struct DeleteMessage {
    pub channel: String,
    pub message: String,
}

impl Component for DeleteMessage {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        Dialog::new()
            .title(label().line_height(1.5).text("Delete message"))
            .body("Are you sure you want to delete this?")
            .default_action("Cancel")
            .action("Delete", {
                let channel = self.channel.clone();
                let message = self.message.clone();
                move || {
                    spawn({
                        let channel = channel.clone();
                        let message = message.clone();
                        async move {
                            http().delete_message(&channel, &message).await.unwrap();
                            modals.write().pop_modal();
                        }
                    });

                    false
                }
            })
    }
}
