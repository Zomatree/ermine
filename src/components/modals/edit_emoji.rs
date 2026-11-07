use freya::{prelude::*, radio::use_radio};
use stoat_models::v0;

use crate::{
    AppChannel, components::{Dialog, SingleLineEntry, use_modals}, consume_material_theme, format_error, http,
};

#[derive(PartialEq)]
pub struct EditEmoji {
    pub id: String,
}

impl Component for EditEmoji {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let radio = use_radio(AppChannel::Emojis);

        let name = use_state(|| radio.read().emojis.get(&self.id).unwrap().name.clone());
        let mut error = use_state(|| None);

        Dialog::new()
            .title(label().line_height(1.5).text("Rename emoji"))
            .body(
                rect()
                    .child(SingleLineEntry::new("Emoji Name", name).max_length(32))
                    .maybe_child(
                        error
                            .read()
                            .clone()
                            .map(|error| label().text(error).color(theme.md.error.as_argb_u32())),
                    ),
            )
            .default_action("Cancel")
            .action("Change", {
                let id = self.id.clone();

                move || {
                    let id = id.clone();
                    let name = name.read().cloned();

                    spawn(async move {
                        match http()
                            .edit_emoji(&id, &v0::DataEditEmoji { name: Some(name) })
                            .await
                        {
                            Ok(_) => {
                                modals.write().pop_modal();
                            }
                            Err(e) => {
                                error.set(Some(format_error(&e, "emoji")));
                            }
                        }
                    });

                    false
                }
            })
    }
}
