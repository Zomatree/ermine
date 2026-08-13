use freya::prelude::*;

use crate::{
    components::{ModalValue, use_modals},
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct Hyperlink {
    pub span: Span<'static>,
}

impl Component for Hyperlink {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();
        let mut hover = use_state(|| false);

        paragraph()
            .line_height(1.5)
            .span(
                self.span
                    .clone()
                    .color(theme.md.primary.as_argb_u32())
                    .text_decoration(if hover() {
                        TextDecoration::Underline
                    } else {
                        TextDecoration::None
                    }),
            )
            .on_press({
                let url = self.span.text.to_string();

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
