use freya::prelude::*;
use stoat_models::v0;

use crate::{
    components::{Dialog, stoat_label},
    http,
};

#[derive(PartialEq)]
pub struct MarkChannelMature {
    pub id: String,
}

impl Component for MarkChannelMature {
    fn render(&self) -> impl IntoElement {
        Dialog::new()
            .title(stoat_label().text("Mark this channel as mature?"))
            .body(
                stoat_label()
                    .text("Users will be asked to confirm their age before joining this channel."),
            )
            .default_action("Keep as is")
            .action("Mark as mature", {
                let id = self.id.clone();

                move || {
                    let id = id.clone();

                    spawn_in_window(async move {
                        http()
                            .edit_channel(
                                &id,
                                &v0::DataEditChannel {
                                    name: None,
                                    description: None,
                                    owner: None,
                                    icon: None,
                                    nsfw: Some(true),
                                    archived: None,
                                    voice: None,
                                    slowmode: None,
                                    remove: Vec::new(),
                                },
                            )
                            .await
                            .unwrap();
                    });

                    true
                }
            })
    }
}
