use freya::{prelude::*, radio::use_radio};

use crate::{
    AppChannel,
    components::{Dialog, Message, MessageModel, use_modals},
    http,
};

#[derive(PartialEq)]
pub struct DeleteMessage {
    pub channel: String,
    pub message: MessageModel,
}

impl Component for DeleteMessage {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();

        let radio = use_radio(AppChannel::Channels);
        let channel = radio.slice_current({
            let channel = self.channel.clone();
            move |state| state.channels.get(&channel).unwrap()
        });

        Dialog::new()
            .title(label().line_height(1.5).text("Delete message"))
            .body(
                rect()
                    .spacing(12.)
                    .child("Are you sure you want to delete this?")
                    .child(
                        rect()
                            .padding((0., 15., 0., 0.))
                            .child(Message {
                                channel: channel.into_readable(),
                                message: self.message.clone(),
                                plain: true,
                            }),
                    ),
            )
            .default_action("Cancel")
            .action("Delete", {
                let channel = self.channel.clone();
                let message_id = self.message.message.id.clone();

                move || {
                    spawn({
                        let channel = channel.clone();
                        let message_id = message_id.clone();
                        async move {
                            http().delete_message(&channel, &message_id).await.unwrap();
                            modals.write().pop_modal();
                        }
                    });

                    false
                }
            })
    }
}
