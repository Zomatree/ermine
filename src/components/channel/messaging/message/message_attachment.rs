use freya::prelude::*;
use stoat_models::v0;

use crate::{
    SizeExt,
    components::{
        MaterialIcon, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt, file_image,
        material::filled::{description, download},
    },
    consume_material_theme, http,
};

#[derive(PartialEq)]
pub struct MessageAttachment {
    pub file: v0::File,
}

impl Component for MessageAttachment {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut spoilered = use_state(|| self.file.filename.starts_with("SPOILER_"));

        rect()
            .corner_radius(12.)
            .overflow(Overflow::Clip)
            .child(match self.file.metadata {
                v0::Metadata::Image {
                    width,
                    height,
                    thumbhash: _,
                    ..
                } => {
                    let new_width = width.min(420) as f32;
                    let new_height = (new_width / width as f32) * height as f32;

                    rect()
                        .width(Size::px(new_width))
                        .height(Size::px(new_height))
                        .child(
                            file_image(&self.file)
                                .width(Size::Fill)
                                .height(Size::Fill)
                                .aspect_ratio(AspectRatio::Min)
                                .image_cover(ImageCover::Fill), // .map(thumbhash.as_ref(), |this, thumbnail| {
                                                                //     this.loading_placeholder(
                                                                //         ImageViewer::new(ImageSource::Bytes(
                                                                //             0,
                                                                //             Bytes::copy_from_slice(thumbnail),
                                                                //         ))
                                                                //         .width(Size::Fill)
                                                                //         .height(Size::Fill)
                                                                //         .aspect_ratio(AspectRatio::Min)
                                                                //         .image_cover(ImageCover::Fill),
                                                                //     )
                                                                // }),
                        )
                        .maybe_child(spoilered.read().then(|| {
                            rect()
                                .width(Size::px(new_width))
                                .height(Size::px(new_height))
                                .position(Position::new_absolute())
                                .layer(Layer::Relative(1))
                                .child(
                                    StoatButton::new()
                                        .on_press(move |_| spoilered.set(false))
                                        .child(
                                            rect()
                                                .width(Size::px(new_width))
                                                .height(Size::px(new_height))
                                                .blur(24.)
                                                .background(0x33FFFFFF)
                                                .center()
                                                .child(
                                                    rect()
                                                        .background(
                                                            theme.md.inverse_surface.as_argb_u32(),
                                                        )
                                                        .color(
                                                            theme
                                                                .md
                                                                .inverse_on_surface
                                                                .as_argb_u32(),
                                                        )
                                                        .corner_radius(16.)
                                                        .padding(8.)
                                                        .child(
                                                            label()
                                                                .font_weight(FontWeight::SEMI_BOLD)
                                                                .line_height(1.25)
                                                                .text("Click to show spoiler"),
                                                        ),
                                                ),
                                        ),
                                )
                        }))
                        .into_element()
                }
                _ => rect()
                    .padding(8.)
                    .corner_radius(12.)
                    .overflow(Overflow::Clip)
                    .width(Size::Fill)
                    .max_width(Size::px(420.))
                    .background(theme.md.inverse_surface.as_argb_u32())
                    .color(theme.md.inverse_on_surface.as_argb_u32())
                    .horizontal()
                    .cross_align(Alignment::Center)
                    .spacing(8.)
                    .content(Content::Flex)
                    .child(MaterialIcon::new(description()).size(Size::px(24.)))
                    .child(
                        rect()
                            .width(Size::flex(1.))
                            .spacing(8.)
                            .child(label().text(self.file.filename.clone()).font_size(14))
                            .child(
                                label()
                                    .text(format!("{:.2} KB", self.file.size as f32 / 1000.))
                                    .font_size(11),
                            ),
                    )
                    .child(
                        StoatButton::new()
                            .corner_radius(20.)
                            .color(theme.md.inverse_on_surface.as_argb_u32())
                            .on_press({
                                let file = self.file.clone();
                                move |_| {
                                    let url = format!(
                                        "{}/{}/{}",
                                        http().api_config.features.autumn.url,
                                        &file.tag,
                                        &file.id
                                    );

                                    open::that_in_background(url);
                                }
                            })
                            .child(
                                rect()
                                    .width(Size::px(40.))
                                    .height(Size::px(40.))
                                    .center()
                                    .child(MaterialIcon::new(download()).size(Size::px(24.))),
                            ),
                    )
                    .into_element(),
            })
    }

    fn render_key(&self) -> DiffKey {
        (&self.file.id).into()
    }
}
