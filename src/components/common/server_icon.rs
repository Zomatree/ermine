use freya::prelude::*;
use stoat_models::v0::Server;

use crate::{components::file_image, theme::Theme};

pub fn server_icon(server: &Server, theme: &Theme) -> impl IntoElement {
    rect().child(match &server.icon {
        Some(file) => file_image(file).into_element(),
        None => rect()
            .background(theme.md.surface_container_low.as_argb_u32())
            .width(Size::Fill)
            .height(Size::Fill)
            .center()
            .child(&server.name[0..1])
            .color(theme.md.on_surface.as_argb_u32())
            .into_element(),
    })
}
