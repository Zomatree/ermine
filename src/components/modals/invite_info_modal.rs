use freya::prelude::*;

use crate::components::Dialog;

#[derive(PartialEq)]
pub struct InviteInfoModal {
    pub code: String,
}

impl Component for InviteInfoModal {
    fn render(&self) -> impl IntoElement {
        let url = format!("https://stt.gg/{}", self.code);

        Dialog::new()
            .title(label().line_height(1.5).text("Create Invite"))
            .body(
                rect().child("Here is your new invite code:").child(
                    rect().padding(15.).child(
                        label()
                            .text(url.clone())
                            .font_family("monospace")
                            .font_size(20.),
                    ),
                ),
            )
            .action("Copy Link", move || {
                Clipboard::set(url.clone()).unwrap();
                false
            })
            .default_action("Ok")
    }
}
