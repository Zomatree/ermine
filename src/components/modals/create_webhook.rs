use freya::prelude::*;
use stoat_models::v0;

use crate::{
    components::{Dialog, SingleLineEntry, use_modals},
    consume_material_theme, http,
};

#[derive(PartialEq)]
pub struct CreateWebhook {
    pub channel: String,
    pub callback: EventHandler<v0::Webhook>,
}

impl Component for CreateWebhook {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();
        let theme = consume_material_theme();
        let name = use_state(String::new);
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(2.).text("Create Webhook"))
            .body(
                rect()
                    .spacing(8.)
                    .child(SingleLineEntry::new("Webhook Name", name))
                    .maybe_child(
                        error
                            .read()
                            .clone()
                            .map(|error| label().text(error).color(theme.md.error.as_argb_u32())),
                    ),
            )
            .default_action("Close")
            .action("Create", {
                let channel = self.channel.clone();
                let callback = self.callback.clone();

                move || {
                    spawn({
                        let name = name.read().clone();
                        let channel = channel.clone();
                        let callback = callback.clone();

                        async move {
                            match http()
                                .create_webhook(
                                    &channel,
                                    &v0::CreateWebhookBody { name, avatar: None },
                                )
                                .await
                            {
                                Ok(response) => {
                                    callback.call(response);
                                    modals.write().pop_modal();
                                }
                                Err(e) => error.set(Some(format!("{e:?}"))),
                            };
                        }
                    });

                    false
                }
            })
    }
}
