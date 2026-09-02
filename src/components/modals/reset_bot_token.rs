use freya::prelude::*;
use stoat_models::v0;

use crate::{
    components::{Dialog, use_modals},
    http,
};

#[derive(PartialEq)]
pub struct ResetBotToken {
    pub id: String,
    pub name: String,
    pub callback: EventHandler<v0::BotWithUserResponse>,
}

impl Component for ResetBotToken {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        Dialog::new()
            .title(
                label()
                    .line_height(1.5)
                    .text(format!("Reset {}'s token?", &self.name)),
            )
            .body(
                label()
                    .line_height(1.5)
                    .text("This will invalidate the current token and stop any existing instances of the bot from running."),
            )
            .default_action("Cancel")
            .action("Reset", {
                let id = self.id.clone();
                let callback = self.callback.clone();

                move || {
                    let id = id.clone();
                    let callback = callback.clone();

                    spawn(async move {
                        if let Ok(response) = http().edit_bot(&id, &v0::DataEditBot {
                            name: None,
                            public: None,
                            analytics: None,
                            interactions_url: None,
                            remove: vec![v0::FieldsBot::Token],
                        }).await {
                            callback.call(response);
                            modals.write().pop_modal();
                        }
                    });

                    false
                }
            })
    }
}
