use freya::prelude::*;
use stoat_models::v0;

use crate::{components::file_image, consume_material_theme};

#[derive(PartialEq)]
pub struct ServerIcon {
    server: Readable<v0::Server>,
}

impl ServerIcon {
    pub fn new(server: impl IntoReadable<v0::Server>) -> Self {
        Self {
            server: server.into_readable()
        }
    }
}

impl Component for ServerIcon {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let server = self.server.read();

        match &server.icon {
            Some(file) => file_image(file).into_element(),
            None => rect()
                .background(theme.md.surface_container_low.as_argb_u32())
                .width(Size::Fill)
                .height(Size::Fill)
                .center()
                .child(
                    server
                        .name
                        .split_whitespace()
                        .take(2)
                        .map(|word| &word[0..1])
                        .collect::<String>(),
                )
                .color(theme.md.on_surface.as_argb_u32())
                .into_element(),
        }
    }
}
