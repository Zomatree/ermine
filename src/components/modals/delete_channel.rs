use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel,
    components::{Dialog, use_modals},
    http,
};

#[derive(PartialEq)]
pub struct DeleteChannel {
    pub channel: String,
}

impl Component for DeleteChannel {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Channels);

        let channel = radio.slice_current({
            let channel = self.channel.clone();
            move |state| state.channels.get(&channel).unwrap()
        });

        let mut modals = use_modals();

        Dialog::new()
            .title(
                label()
                    .line_height(1.5)
                    .text(format!("Delete {}?", channel.read().name().unwrap())),
            )
            .body("Once it's deleted, there's no going back.")
            .default_action("Cancel")
            .action("Delete", {
                let channel = self.channel.clone();

                move || {
                    spawn({
                        let channel = channel.clone();
                        async move {
                            modals.write().pop_modal();

                            http().delete_channel(&channel).await.unwrap();
                        }
                    });

                    false
                }
            })
    }
}
