use freya::prelude::*;

use crate::{
    components::{
        markdown::{parser::Inline, render::render_content},
        stoat_paragraph,
    },
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct Spoiler {
    pub content: Vec<Inline>,
    pub font_size: f32,
    pub foreground: Color,
    pub link_foreground: Color,
}

impl Component for Spoiler {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut show = use_state(|| false);
        let holder = use_hook(ParagraphHolder::default);

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
                    .child(
                        render_content(
                            stoat_paragraph(),
                            &self.content,
                            self.font_size,
                            self.foreground,
                            self.link_foreground,
                            false,
                            false,
                            false,
                        )
                        .holder(holder),
                    ),
            )
    }
}
