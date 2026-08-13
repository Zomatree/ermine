use freya::prelude::*;

use crate::components::Dialog;

#[derive(PartialEq)]
pub struct OpenLink {
    pub url: String,
}

impl Component for OpenLink {
    fn render(&self) -> impl IntoElement {
        Dialog::new()
            .title(label().line_height(1.5).text("External links can be dangerous!"))
            .body(
                rect()
                    .child(
                        label()
                            .line_height(1.5)
                            .text("Are you sure you want to go to"),
                    )
                    .child(label().text_decoration(TextDecoration::Underline).text(self.url.clone())),
            )
            .default_action("Close")
            .action("Continue", {let url = self.url.clone(); move || {
                open::that_in_background(&url);
                true
            }})
    }
}
