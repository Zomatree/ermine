use freya::prelude::*;

use crate::components::{
    markdown::{
        MarkdownColours,
        components::{
            ChannelMention, Emoji, Hyperlink, Latex, Link, MessageMention, RoleMention, Spoiler,
            UserMention,
        },
        parser::{Inline, List},
    },
    stoat_label, stoat_paragraph,
};

pub fn render_content(
    mut p: Paragraph,
    content: &[Inline],
    base_font_size: f32,
    colours: MarkdownColours,
    bold: bool,
    link: bool,
    underline: bool,
) -> Paragraph {
    for item in content {
        p = match item {
            Inline::Span(span) => {
                if span.code {
                    p.child(
                        rect()
                            .background(0xff0d1117)
                            .padding((1., 4.))
                            .corner_radius(12.)
                            .child(
                                stoat_label()
                                    .color(0xffc9d1d9)
                                    .line_height(1.5)
                                    .font_family("Fira Code")
                                    .text(span.text.clone()),
                            ),
                    )
                } else {
                    let mut styled = Span::new(span.text.replace('\t', " ").clone())
                        .font_size(base_font_size)
                        .color(colours.foreground);

                    if span.bold || bold {
                        styled = styled.font_weight(FontWeight::BOLD);
                    }
                    if span.italic {
                        styled = styled.font_slant(FontSlant::Italic);
                    }
                    if span.strikethrough {
                        styled = styled.text_decoration(TextDecoration::LineThrough)
                    }
                    if underline {
                        styled = styled.text_decoration(TextDecoration::Underline)
                    }

                    if span.link {
                        p.child(Hyperlink {
                            span: styled,
                            font_size: base_font_size,
                            colours,
                        })
                    } else {
                        if link {
                            styled = styled.color(colours.link_foreground)
                        };

                        p.span(styled)
                    }
                }
            }
            Inline::Link { text, url, title } => p.child(Link {
                content: text.clone(),
                url: url.clone(),
                title: title.clone(),
                bold,
                font_size: base_font_size,
                colours,
            }),
            Inline::UserMention { id } => p.child(UserMention {
                id: id.clone(),
                font_size: base_font_size,
            }),
            Inline::ChannelMention { id } => p.child(ChannelMention {
                id: id.clone(),
                font_size: base_font_size,
            }),
            Inline::RoleMention { id } => p.child(RoleMention {
                id: id.clone(),
                font_size: base_font_size,
            }),
            Inline::MessageMention { channel_id, id } => p.child(MessageMention {
                id: id.clone(),
                channel_id: channel_id.clone(),
                font_size: base_font_size,
            }),
            Inline::Emoji { id } => p.child(Emoji {
                id: id.clone(),
                font_size: base_font_size,
            }),
            Inline::Spoiler(text) => p.child(Spoiler {
                content: text.clone(),
                font_size: base_font_size,
                colours,
            }),
            Inline::Latex(content) => p.child(Latex {
                content: content.clone(),
                font_size: base_font_size,
                colours,
            }),
        };
    }

    p.line_height(1.5)
}

pub fn render_list(
    list: &List,
    base_font_size: f32,
    colours: MarkdownColours,
    bold: bool,
    link: bool,
    underline: bool,
) -> Rect {
    rect()
        .vertical()
        .children(list.items.iter().enumerate().map(|(item_idx, item)| {
            rect()
                .key(item_idx)
                .horizontal()
                .cross_align(Alignment::Start)
                .spacing(8.)
                .child(
                    stoat_label()
                        .color(colours.foreground)
                        .line_height(1.5)
                        .text(match list.start {
                            Some(start) => format!("{}.", start + item_idx as u64),
                            None => "•".to_string(),
                        })
                        .font_size(base_font_size),
                )
                .child(
                    rect()
                        .vertical()
                        .child(render_content(
                            stoat_paragraph(),
                            &item.content,
                            base_font_size,
                            colours,
                            bold,
                            link,
                            underline,
                        ))
                        .children(item.nested_lists.iter().map(|nested_list| {
                            render_list(nested_list, base_font_size, colours, bold, link, underline)
                        })),
                )
        }))
}
