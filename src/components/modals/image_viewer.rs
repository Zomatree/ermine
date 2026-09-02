use freya::prelude::*;

use crate::{
    SizeExt,
    components::{
        AnimatedImage, MaterialIcon, StoatButton, StoatButtonLayoutThemePartialExt,
        material::outlined::{close, open_in_new},
        use_modals,
    },
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct ImageViewer {
    pub content: Url,
}

impl Component for ImageViewer {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        rect()
            .expanded()
            .center()
            .child(AnimatedImage::new(self.content.clone()).image_cover(ImageCover::Center))
            .child(
                rect()
                    .horizontal()
                    .position(Position::new_absolute().top(32.).right(32.))
                    .padding(8.)
                    .background(theme.md.surface.as_argb_u32())
                    .color(theme.md.on_surface_variant.as_argb_u32())
                    .corner_radius(16.)
                    .layer(10)
                    .spacing(8.)
                    .child(
                        StoatButton::new()
                            .corner_radius(20.)
                            .on_press({
                                let url = self.content.clone();

                                move |_| {
                                    open::that_in_background(url.as_str());
                                }
                            })
                            .child(
                                rect()
                                    .size(Size::px(40.))
                                    .center()
                                    .child(MaterialIcon::new(open_in_new()).size(Size::px(24.))),
                            ),
                    )
                    .child(
                        StoatButton::new()
                            .corner_radius(20.)
                            .on_press(move |_| {
                                modals.write().pop_modal();
                            })
                            .child(
                                rect()
                                    .size(Size::px(40.))
                                    .center()
                                    .child(MaterialIcon::new(close()).size(Size::px(24.))),
                            ),
                    ),
            )
    }
}
