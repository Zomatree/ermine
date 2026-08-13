use freya::prelude::*;

use crate::{
    components::markdown::{
        components::{ChannelMention, Emoji, Hyperlink, Link, RoleMention, Spoiler, UserMention},
        parser::Inline,
    },
    theme::Theme,
};

pub fn render_content(
    mut p: Paragraph,
    content: &[Inline],
    base_font_size: f32,
    theme: Theme,
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
                                label()
                                    .color(0xffc9d1d9)
                                    .line_height(1.5)
                                    .font_family("Fira Code")
                                    .text(span.text.clone()),
                            ),
                    )
                } else {
                    let mut styled = Span::new(span.text.clone()).font_size(base_font_size);

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
                        p.child(Hyperlink { span: styled })
                    } else {
                        if link {
                            styled = styled.color(theme.md.primary.as_argb_u32())
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
            Inline::Emoji { id } => p.child(Emoji {
                id: id.clone(),
                font_size: base_font_size,
            }),
            Inline::Spoiler(text) => p.child(Spoiler {
                content: text.clone(),
                font_size: base_font_size,
            }),
        };
    }

    p.line_height(1.5)
}
