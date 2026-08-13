use freya::prelude::*;

use crate::{
    components::markdown::{parser::Inline, render::render_content},
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct Spoiler {
    pub content: Vec<Inline>,
    pub font_size: f32,
}

impl Component for Spoiler {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut show = use_state(|| false);

        rect()
            .padding((0., 2.))
            .background(
                if show() {
                    theme.md.inverse_surface
                } else {
                    theme.md.surface_container
                }
                .as_argb_u32(),
            )
            .color(
                if show() {
                    theme.md.inverse_on_surface
                } else {
                    theme.md.on_surface
                }
                .as_argb_u32(),
            )
            .corner_radius(12.)
            .on_press(move |_| show.set(true))
            .on_pointer_enter(move |_| {
                Cursor::set(CursorIcon::Pointer);
            })
            .on_pointer_leave(move |_| {
                Cursor::set(CursorIcon::default());
            })
            .child(
                rect()
                    .opacity(if show() { 1. } else { 0. })
                    .child(render_content(
                        paragraph(),
                        &self.content,
                        self.font_size,
                        theme,
                        false,
                        false,
                        false,
                    )),
            )
    }
}
