use freya::prelude::*;
use stoat_models::v0;

use crate::{
    components::{
        AnimatedImage, MarkdownViewer, MessageAttachment, MessageModel, ModalValue, use_modals,
    },
    consume_material_theme, http, parse_fill, proxy_url,
};

#[derive(PartialEq)]
pub struct MessageEmbed {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
    pub embed: v0::Embed,
}

impl Component for MessageEmbed {
    fn render(&self) -> impl IntoElement {
        match self.embed.clone() {
            v0::Embed::Website(metadata) => WebsiteEmbed {
                channel: self.channel.clone(),
                message: self.message.clone(),
                metadata,
            }
            .into_element(),
            v0::Embed::Image(image) => {
                let new_width = image.width.min(420) as f32;
                let new_height = (new_width / image.width as f32) * image.height as f32;

                rect()
                    .width(Size::px(new_width))
                    .height(Size::px(new_height))
                    .child(
                        AnimatedImage::new(proxy_url(&image.url))
                            .sampling_mode(SamplingMode::Trilinear)
                            .width(Size::Fill)
                            .height(Size::Fill)
                            .aspect_ratio(AspectRatio::Min)
                            .image_cover(ImageCover::Fill)
                            .corner_radius(12.)
                            .selectable(true),
                    )
                    .into_element()
            }
            v0::Embed::Video(_) => "<Video>".into_element(),
            v0::Embed::Text(text) => TextEmbed {
                channel: self.channel.clone(),
                message: self.message.clone(),
                text,
            }
            .into_element(),
            v0::Embed::None => unreachable!(),
        }
    }
}

#[derive(PartialEq)]
pub struct WebsiteEmbed {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
    pub metadata: v0::WebsiteMetadata,
}

impl Component for WebsiteEmbed {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let mut hover_title = use_state(|| false);

        let border_color = use_hook(|| {
            self.metadata
                .colour
                .as_deref()
                .and_then(parse_fill)
                .and_then(|fill| {
                    if let Fill::Color(color) = fill {
                        Some(color)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| theme.md.primary.as_argb_u32().into())
        });

        rect()
            .corner_radius(12.)
            .overflow(Overflow::Clip)
            .background(theme.md.primary_container.as_argb_u32())
            .color(theme.md.on_primary_container.as_argb_u32())
            .max_width(Size::px(420.))
            .border(Border::new().fill(border_color).width(BorderWidth {
                top: 0.,
                right: 0.,
                bottom: 0.,
                left: 4.,
            }))
            .padding((8., 8., 8., 12.))
            .spacing(8.)
            .maybe_child(
                (self.metadata.site_name.is_some() && self.metadata.icon_url.is_some()).then(
                    || {
                        rect()
                            .horizontal()
                            .spacing(8.)
                            .cross_align(Alignment::Center)
                            .maybe_child(self.metadata.icon_url.as_ref().map(|url| {
                                AnimatedImage::new(proxy_url(url))
                                    .width(Size::px(14.))
                                    .height(Size::px(14.))
                            }))
                            .maybe_child(self.metadata.site_name.clone().map(|site_name| {
                                label()
                                    .max_lines(1)
                                    .text_overflow(TextOverflow::Ellipsis)
                                    .font_size(11.)
                                    .text(site_name)
                            }))
                    },
                ),
            )
            .maybe_child(self.metadata.title.clone().map(|title| {
                rect()
                    .on_pointer_enter(move |_| hover_title.set_if_modified(true))
                    .on_pointer_out(move |_| hover_title.set_if_modified(false))
                    .maybe(self.metadata.url.is_some(), move |this| {
                        this.cursor(CursorIcon::Pointer)
                    })
                    .child(
                        label()
                            .font_size(16.)
                            .max_lines(1)
                            .text_overflow(TextOverflow::Ellipsis)
                            .text(title)
                            .map(self.metadata.url.clone(), |label, url| {
                                label
                                    .text_decoration(if hover_title() {
                                        TextDecoration::Underline
                                    } else {
                                        TextDecoration::None
                                    })
                                    .color(theme.md.primary.as_argb_u32())
                                    .on_press(move |_| {
                                        modals
                                            .write()
                                            .push_modal(ModalValue::OpenLink { url: url.clone() });
                                    })
                            }),
                    )
            }))
            .maybe_child(
                self.metadata
                    .description
                    .clone()
                    .map(|description| MarkdownViewer::new(description, None).font_size(12.)),
            )
            .maybe_child(self.metadata.image.clone().map(|image| {
                let new_width = image.width.min(400) as f32;
                let new_height = (new_width / image.width as f32) * image.height as f32;

                rect()
                    .width(Size::px(new_width))
                    .height(Size::px(new_height))
                    .corner_radius(12.)
                    .overflow(Overflow::Clip)
                    .child(
                        AnimatedImage::new(proxy_url(&image.url))
                            .sampling_mode(SamplingMode::Trilinear)
                            .width(Size::Fill)
                            .height(Size::Fill)
                            .aspect_ratio(AspectRatio::Min)
                            .image_cover(ImageCover::Fill)
                            .corner_radius(12.)
                            .selectable(true),
                    )
            }))
    }
}

#[derive(PartialEq)]
pub struct TextEmbed {
    pub channel: Readable<v0::Channel>,
    pub message: MessageModel,
    pub text: v0::Text,
}

impl Component for TextEmbed {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut modals = use_modals();

        let mut hover_title = use_state(|| false);

        let border_color = use_hook(|| {
            self.text
                .colour
                .as_deref()
                .and_then(parse_fill)
                .and_then(|fill| {
                    if let Fill::Color(color) = fill {
                        Some(color)
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| theme.md.primary.as_argb_u32().into())
        });

        rect()
            .corner_radius(12.)
            .overflow(Overflow::Clip)
            .background(theme.md.primary_container.as_argb_u32())
            .color(theme.md.on_primary_container.as_argb_u32())
            .max_width(Size::px(420.))
            .border(Border::new().fill(border_color).width(BorderWidth {
                top: 0.,
                right: 0.,
                bottom: 0.,
                left: 4.,
            }))
            .padding((8., 8., 8., 12.))
            .spacing(8.)
            .maybe_child(self.text.title.clone().map(|title| {
                rect()
                    .on_pointer_enter(move |_| hover_title.set_if_modified(true))
                    .on_pointer_out(move |_| hover_title.set_if_modified(false))
                    .maybe(self.text.url.is_some(), |this| {
                        this.cursor(CursorIcon::Pointer)
                            .text_decoration(if hover_title() {
                                TextDecoration::Underline
                            } else {
                                TextDecoration::None
                            })
                    })
                    .child(
                        label()
                            .font_size(16.)
                            .max_lines(1)
                            .text_overflow(TextOverflow::Ellipsis)
                            .text(title)
                            .map(self.text.url.clone(), |label, url| {
                                label
                                    .color(theme.md.primary.as_argb_u32())
                                    .on_press(move |_| {
                                        modals
                                            .write()
                                            .push_modal(ModalValue::OpenLink { url: url.clone() });
                                    })
                            }),
                    )
            }))
            .maybe_child(
                self.text
                    .description
                    .clone()
                    .map(|description| MarkdownViewer::new(description, None).font_size(12.)),
            )
            .maybe_child(
                self.text
                    .media
                    .clone()
                    .map(|file| MessageAttachment { file }),
            )
    }
}
