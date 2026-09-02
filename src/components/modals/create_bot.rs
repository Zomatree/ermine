use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, SelectedRole, ServerSettingsPage,
    components::{Dialog, SingleLineEntry, use_modals},
    consume_material_theme, http,
};

#[derive(PartialEq)]
pub struct CreateBot {
    pub callback: EventHandler<v0::BotWithUserResponse>,
}

impl Component for CreateBot {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();
        let theme = consume_material_theme();
        let name = use_state(String::new);
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(2.).text("Create a new bot"))
            .body(
                rect()
                    .spacing(8.)
                    .child("By creating this bot, you agree to the Acceptable Use Policy.")
                    .child(SingleLineEntry::new("Username", name))
                    .maybe_child(
                        error
                            .read()
                            .clone()
                            .map(|error| label().text(error).color(theme.md.error.as_argb_u32())),
                    ),
            )
            .default_action("Close")
            .action("Create", {
                let callback = self.callback.clone();

                move || {
                    spawn({
                        let callback = callback.clone();
                        let name = name.read().clone();

                        async move {
                            match http().create_bot(&v0::DataCreateBot { name }).await {
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
