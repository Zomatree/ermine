use freya::prelude::*;

use crate::components::{UserMention as BaseUserMention, markdown::components::consume_server};

#[derive(PartialEq)]
pub struct UserMention {
    pub id: String,
    pub font_size: f32,
}

impl Component for UserMention {
    fn render(&self) -> impl IntoElement {
        let server = consume_server();

        BaseUserMention::new(self.id.clone(), server).font_size(self.font_size)
    }
}
