use freya::prelude::*;

use crate::{
    components::markdown::components::consume_server,
    consume_material_theme, parse_fill,
};

#[derive(PartialEq)]
pub struct RoleMention {
    pub id: String,
    pub font_size: f32,
}

impl Component for RoleMention {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let server = consume_server();

        let (name, fill) = use_hook(|| {
            if let Some(server) = server {
                let server = server.read();

                if let Some(role) = server.roles.get(&self.id) {
                    return (
                        role.name.clone(),
                        role.colour.as_deref().and_then(parse_fill),
                    );
                }
            };

            ("Unknown Role".to_string(), None)
        });

        let size = self.font_size * (16. / 14.);
        let fill =
            fill.unwrap_or_else(|| Fill::Color(theme.md.on_primary_container.as_argb_u32().into()));

        rect()
            .color(fill.clone())
            .padding((0., 6., 0., 2.))
            .horizontal()
            .cross_align(Alignment::Center)
            .spacing(4.)
            .corner_radius(16.)
            .background(theme.md.primary_container.as_argb_u32())
            .font_weight(FontWeight::SEMI_BOLD)
            .child(
                rect()
                    .background(fill)
                    .corner_radius(size / 2.)
                    .width(Size::px(size))
                    .height(Size::px(size)),
            )
            .child(
                label()
                    .line_height(1.5)
                    .max_lines(1)
                    .font_size(self.font_size)
                    .text(name),
            )
    }
}