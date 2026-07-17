use freya::prelude::*;

use crate::{Error, components::Dialog};

#[derive(PartialEq)]
pub struct ErrorModal {
    pub error: Error,
}

impl Component for ErrorModal {
    fn render(&self) -> impl IntoElement {
        Dialog::new()
            .title(label().line_height(1.5).text("An Error Occurred"))
            .body(
                rect()
                    .child(format!("{:?}", self.error))
            )
            .default_action("Ok")
    }
}
