use freya::prelude::*;

use crate::components::{
    ModalValue, StoatTooltip,
    markdown::{MarkdownColours, components::MessageUrl, parser::Inline, render::render_content},
    stoat_paragraph, use_modals,
};

#[derive(PartialEq)]
pub struct Link {
    pub content: Vec<Inline>,
    pub url: String,
    pub title: Option<String>,
    pub bold: bool,
    pub font_size: f32,
    pub colours: MarkdownColours,
}

impl Component for Link {
    fn render(&self) -> impl IntoElement {
        let mut modals = use_modals();
        let mut hover = use_state(|| false);

        let element = render_content(
            stoat_paragraph(),
            &self.content,
            self.font_size,
            MarkdownColours {
                foreground: self.colours.link_foreground,
                ..self.colours
            },
            self.bold,
            true,
            hover(),
        )
        .cursor(CursorIcon::Pointer)
        .on_pointer_over(move |_| {
            hover.set_if_modified(true);
        })
        .on_pointer_out(move |_| {
            hover.set_if_modified(false);
        })
        .on_secondary_down({
            let url = self.url.clone();
            move |_| {
                provide_root_context(Some(MessageUrl(url.clone())));
            }
        })
        .on_press({
            let url = self.url.clone();

            move |_| {
                modals
                    .write()
                    .push_modal(ModalValue::OpenLink { url: url.clone() });
            }
        });

        if let Some(title) = self.title.clone() && !title.is_empty() {
            StoatTooltip::new(label().font_size(11.).max_lines(1).text(title))
                .compact(true)
                .position(AttachedPosition::Top)
                .child(element)
                .into_element()
        } else {
            element.into_element()
        }
    }
}
