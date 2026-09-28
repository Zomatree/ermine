use freya::prelude::*;
use stoat_models::v0;

use crate::components::{Message, MessageActions, MessageModel, ReplyController, TrailingMessage};

#[derive(PartialEq)]
pub struct MessageGroup {
    pub replies: ReplyController,
    pub messages: Vec<MessageModel>,
    pub channel: Readable<v0::Channel>,
}

impl Component for MessageGroup {
    fn render(&self) -> impl IntoElement {
        let first = self.messages.first().unwrap().clone();

        rect()
            .padding((0., 16., 0., 0.))
            .child(
                MessageActions::new(self.replies, self.channel.clone(), first.clone()).child(
                    rect().padding((2., 0.)).child(Message {
                        channel: self.channel.clone(),
                        message: first,
                        plain: false,
                    }),
                ),
            )
            .children(self.messages[1..].iter().cloned().map(|message| {
                println!("{:?}", message.message);
                MessageActions::new(self.replies, self.channel.clone(), message.clone())
                    .child(rect().padding((2., 0.)).child(TrailingMessage {
                        channel: self.channel.clone(),
                        message,
                    }))
                    .into_element()
            }))
    }
}
