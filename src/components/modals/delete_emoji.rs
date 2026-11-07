use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel,
    components::{Dialog, stoat_label},
    http,
};

#[derive(PartialEq)]
pub struct DeleteEmoji {
    pub id: String,
}

impl Component for DeleteEmoji {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Emojis);

        Dialog::new()
            .title(label().line_height(1.5).text(format!(
                "Delete :{}:?",
                radio.read().emojis.get(&self.id).unwrap().name
            )))
            .body(
                stoat_label().text(
                    "Deleting this emoji will disallow future use. All past uses will remain.",
                ),
            )
            .default_action("Cancel")
            .action("Delete", {
                let id = self.id.clone();

                move || {
                    let id = id.clone();

                    spawn_in_window(async move {
                        http().delete_emoji(&id).await.unwrap();
                    });

                    true
                }
            })
    }
}
