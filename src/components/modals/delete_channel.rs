use freya::prelude::*;

use crate::{
    components::{Dialog, use_modals},
    http,
};

#[derive(PartialEq)]
pub struct DeleteChannel {
    pub channel: String,
    pub name: String,
    pub callback: EventHandler<bool>,
}

impl Component for DeleteChannel {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        Dialog::new()
            .title(
                label()
                    .line_height(1.5)
                    .text(format!("Delete {}?", self.name)),
            )
            .body("Once it's deleted, there's no going back.")
            .default_action("Cancel")
            .action("Delete", {
                let channel = self.channel.clone();
                let callback = self.callback.clone();

                move || {
                    spawn({
                        let channel = channel.clone();
                        let callback = callback.clone();

                        async move {
                            modals.write().pop_modal();

                            http().delete_channel(&channel).await.unwrap();
                            callback.call(true)
                        }
                    });

                    false
                }
            })
    }
}
