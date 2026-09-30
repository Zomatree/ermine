use freya::prelude::*;
use stoat_models::v0;

use crate::{SizeExt, components::file_image, consume_material_theme};

#[derive(PartialEq)]
pub struct ServerIcon {
    icon: Option<v0::File>,
    name: String,
    size: f32,

    corner_radius: CornerRadius,
}

impl ServerIcon {
    pub fn new(server: impl IntoReadable<v0::Server>, size: f32) -> Self {
        Self::from_readable(server.into_readable(), size)
    }

    pub fn from_readable(server: Readable<v0::Server>, size: f32) -> Self {
        let server = server.read();

        Self {
            icon: server.icon.clone(),
            name: server.name.clone(),
            size,
            corner_radius: CornerRadius::default(),
        }
    }

    pub fn from_values(icon: Option<v0::File>, name: String, size: f32) -> Self {
        Self {
            icon,
            name,
            size,
            corner_radius: CornerRadius::default(),
        }
    }

    pub fn corner_radius(mut self, corner_radius: impl Into<CornerRadius>) -> Self {
        self.corner_radius = corner_radius.into();
        self
    }
}

impl Component for ServerIcon {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        match &self.icon {
            Some(file) => file_image(file)
                .size(Size::px(self.size))
                .corner_radius(self.corner_radius)
                .into_element(),
            None => rect()
                .size(Size::px(self.size))
                .center()
                .background(theme.md.surface_container_low.as_argb_u32())
                .color(theme.md.on_surface.as_argb_u32())
                .font_size((12. / 32.) * self.size)
                .corner_radius(self.corner_radius)
                .child(
                    self.name
                        .split_whitespace()
                        .take(2)
                        .map(|word| &word[0..1])
                        .collect::<String>(),
                )
                .into_element(),
        }
    }
}
