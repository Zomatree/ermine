use freya::prelude::*;

use crate::{
    components::{
        ModalValue,
        markdown::{parser::Inline, render::render_content},
        use_modals,
    },
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct Link {
    pub content: Vec<Inline>,
    pub url: String,
    pub title: Option<String>,
    pub bold: bool,
    pub font_size: f32,
}

impl Component for Link {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();
        let mut hover = use_state(|| false);

        render_content(
            paragraph(),
            &self.content,
            self.font_size,
            theme,
            self.bold,
            true,
            hover(),
        )
        // .text_decoration(if hover() { TextDecoration::Underline } else { TextDecoration::None })
        .on_press({
            let url = self.url.clone();

            move |_| {
                modals
                    .write()
                    .push_modal(ModalValue::OpenLink { url: url.clone() });
            }
        })
        .on_pointer_enter(move |_| {
            hover.set_if_modified(true);
            Cursor::set(CursorIcon::Pointer);
        })
        .on_pointer_leave(move |_| {
            hover.set_if_modified(false);
            Cursor::set(CursorIcon::default());
        })
    }
}
