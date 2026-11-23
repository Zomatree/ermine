use freya::prelude::*;

use crate::{
    components::{
        markdown::{MarkdownColours, parser::Inline, render::render_content}, stoat_paragraph,
    }, consume_material_theme,
};

#[derive(PartialEq)]
pub struct Spoiler {
    pub content: Vec<Inline>,
    pub font_size: f32,
    pub colours: MarkdownColours,
}

impl Component for Spoiler {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut show = use_state(|| false);

        rect()
            .padding((0., 6.))
            .background(if show() {
                theme.md.inverse_on_surface
            } else {
                theme.md.on_secondary_fixed_variant
            })
            .corner_radius(12.)
            .on_press(move |_| show.set(true))
            .cursor(if show() {
                None
            } else {
                Some(CursorIcon::Pointer)
            })
            .child(
                rect()
                    .opacity(if show() { 1. } else { 0. })
                    .interactive(show())
                    .child(render_content(
                        stoat_paragraph(),
                        &self.content,
                        self.font_size,
                        self.colours,
                        false,
                        false,
                        false,
                    )),
            )
    }
}
