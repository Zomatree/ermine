use freya::prelude::*;
use itertools::Itertools;

use crate::components::{Dialog, stoat_label};

#[derive(PartialEq)]
pub struct ViewRecoveryCodes {
    pub codes: Vec<String>,
}

impl Component for ViewRecoveryCodes {
    fn render(&self) -> impl IntoElement {
        Dialog::new()
            .title(label().line_height(1.5).text("Your recovery codes"))
            .body(
                rect()
                    .child(
                        stoat_label()
                            .width(Size::Fill)
                            .text_align(TextAlign::Center)
                            .text("Please save these to a safe location."),
                    )
                    .child(
                        rect().children(self.codes.iter().chunks(2).into_iter().map(|codes| {
                            rect()
                                .content(Content::Flex)
                                .horizontal()
                                .children(codes.map(|code| {
                                    stoat_label()
                                        .text(code.clone())
                                        .width(Size::flex(1.))
                                        .text_align(TextAlign::Center)
                                }))
                        })),
                    ),
            )
            .action("Copy", {
                let codes = self.codes.clone();

                move || {
                    Clipboard::set(codes.join("\n")).unwrap();

                    false
                }
            })
            .default_action("Done")
    }
}
