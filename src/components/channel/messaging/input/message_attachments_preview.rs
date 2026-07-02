use freya::{
    icons::lucide::{circle_x, eye, eye_off, file_text, plus},
    prelude::*,
};

use crate::{
    components::{
        Attachment, AttachmentController, StoatButton, StoatButtonColorsThemePartialExt,
        StoatButtonLayoutThemePartialExt,
    },
    use_material_theme,
};

#[derive(PartialEq)]
pub struct MessageAttachmentsPreview {
    pub attachments: AttachmentController,
}

impl Component for MessageAttachmentsPreview {
    fn render(&self) -> impl IntoElement {
        let theme = use_material_theme();

        rect()
            .horizontal()
            .padding(8.)
            .corner_radius(16.)
            .background(theme.md.primary_container.as_argb_u32())
            .width(Size::Fill)
            .child(
                ScrollView::new()
                    .height(Size::px(130.))
                    .spacing(8.)
                    .direction(Direction::Horizontal)
                    .children(
                        self.attachments.get_attachments().map(|attachment| {
                            MessageAttachmentPreview { attachment }.into_element()
                        }),
                    )
                    .child(
                        StoatButton::new()
                            .corner_radius(8.)
                            .color(theme.md.on_primary_container.as_argb_u32())
                            .on_press({
                                let attachments = self.attachments.clone();

                                move |_| {
                                    spawn(async move { attachments.prompt().await });
                                }
                            })
                            .child(
                                rect()
                                    .width(Size::px(100.))
                                    .height(Size::px(100.))
                                    .center()
                                    .child(svg(plus()).width(Size::px(48.)).height(Size::px(48.))),
                            ),
                    )
                    .child(rect()),
            )
    }
}

#[derive(PartialEq)]
pub struct MessageAttachmentPreview {
    pub attachment: Attachment,
}

impl Component for MessageAttachmentPreview {
    fn render(&self) -> impl IntoElement {
        let is_image = use_hook(|| infer::is_image(&self.attachment.contents));
        let mut hovering = use_state(|| false);
        let area = use_state(Area::default);
        let theme = use_material_theme();

        rect()
            .cross_align(Alignment::Center)
            .color(theme.md.on_primary_container.as_argb_u32())
            .child(
                Button::new()
                    .flat()
                    .height(Size::px(100.))
                    .background(Color::TRANSPARENT)
                    .hover_background(Color::TRANSPARENT)
                    .border_fill(Color::TRANSPARENT)
                    .padding(0.)
                    .on_press({
                        let attachment = self.attachment.clone();

                        move |_| {
                            attachment.remove();
                        }
                    })
                    .corner_radius(8.)
                    .child(
                        rect()
                            .height(Size::px(100.))
                            .on_pointer_over(move |_| {
                                hovering.set(true);
                            })
                            .on_pointer_out(move |_| hovering.set_if_modified(false))
                            .child(
                                rect()
                                    .child(if is_image {
                                        ImageViewer::new(ImageSource::Bytes(
                                            self.attachment.id,
                                            self.attachment.contents.clone(),
                                        ))
                                        .into_element()
                                    } else {
                                        rect()
                                            .width(Size::px(100.))
                                            .height(Size::px(100.))
                                            .center()
                                            .child(
                                                svg(file_text())
                                                    .width(Size::px(36.))
                                                    .height(Size::px(36.)),
                                            )
                                            .into_element()
                                    })
                                    .on_sized({
                                        let mut area = area.clone();
                                        move |e: Event<SizedEventData>| area.set(e.area)
                                    }),
                            )
                            .maybe_child(hovering.read().then(|| {
                                rect()
                                    .position(Position::new_absolute())
                                    .width(Size::px(area.read().width()))
                                    .height(Size::px(100.))
                                    .layer(Layer::Relative(1))
                                    .background(0xcc000000)
                                    .center()
                                    .child(
                                        svg(circle_x())
                                            .width(Size::px(36.))
                                            .height(Size::px(36.))
                                            .color(Color::WHITE),
                                    )
                                    .child(
                                        rect()
                                            .position(Position::new_absolute().right(4.).top(4.))
                                            .layer(Layer::Relative(1))
                                            .child(
                                                StoatButton::new()
                                                    .corner_radius(4.)
                                                    .child(
                                                        rect()
                                                            .background(
                                                                theme
                                                                    .md
                                                                    .secondary_container
                                                                    .as_argb_u32(),
                                                            )
                                                            .padding(4.)
                                                            .child(
                                                                svg(if self.attachment.spoiler {
                                                                    eye_off()
                                                                } else {
                                                                    eye()
                                                                })
                                                                .width(Size::px(24.))
                                                                .height(Size::px(24.))
                                                                .color(
                                                                    theme
                                                                        .md
                                                                        .on_secondary_container
                                                                        .as_argb_u32(),
                                                                ),
                                                            ),
                                                    )
                                                    .on_press({
                                                        let attachment = self.attachment.clone();
                                                        move |e: Event<PressEventData>| {
                                                            e.stop_propagation();
                                                            attachment.toggle_spoiler();
                                                        }
                                                    }),
                                            ),
                                    )
                            })),
                    ),
            )
            .child(
                label()
                    .text(self.attachment.filename.clone())
                    .max_lines(1)
                    .text_overflow(TextOverflow::Ellipsis)
                    .font_size(12)
                    .max_width(Size::px(100.)),
            )
            .child(
                label()
                    .text(format!("{} KB", self.attachment.contents.len() / 1000))
                    .font_size(11),
            )
    }

    fn render_key(&self) -> DiffKey {
        (&self.attachment.id).into()
    }
}
