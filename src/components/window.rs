use freya::prelude::*;

use crate::{
    components::{HttpManager, Root, stoat_label},
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct Window {}

impl Component for Window {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        rect()
            .maybe_child(cfg!(target_os = "macos").then(|| {
                rect()
                    .height(Size::px(29.))
                    .width(Size::Fill)
                    .window_drag()
                    .background(theme.md.surface_container_high)
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .main_align(Alignment::End)
                    .child(
                        stoat_label()
                            .padding((0., 8.))
                            .font_size(16.)
                            .font_weight(1000)
                            .font_width(8)
                            .text("Stoat"),
                    )
            }))
            .child(HttpManager::new().child(Root {}))
    }
}
