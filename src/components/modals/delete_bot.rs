use freya::prelude::*;

use crate::{
    components::{Dialog, use_modals},
    http,
};

#[derive(PartialEq)]
pub struct DeleteBot {
    pub id: String,
    pub name: String,
    pub callback: EventHandler<bool>,
}

impl Component for DeleteBot {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        Dialog::new()
            .title(
                label()
                    .line_height(1.5)
                    .text(format!("Delete {}?", &self.name)),
            )
            .body(
                label()
                    .line_height(1.5)
                    .text("Once it's deleted, there's no going back."),
            )
            .default_action("Cancel")
            .action("Delete", {
                let id = self.id.clone();
                let callback = self.callback.clone();

                move || {
                    spawn({
                        let id = id.clone();
                        let callback = callback.clone();
                        async move {
                            if http().delete_bot(&id).await.is_ok() {
                                callback.call(true);
                                modals.write().pop_modal();
                            }
                        }
                    });

                    false
                }
            })
    }
}
