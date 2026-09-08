use std::borrow::Cow;

use freya::prelude::*;
use syntect::{
    easy::HighlightLines,
    util::LinesWithEndings,
};

use crate::{
    components::{StoatButton, StoatButtonLayoutThemePartialExt, StoatTooltip},
    consume_material_theme, use_config, ThemeSet, SyntaxSet
};

#[derive(PartialEq)]
pub struct Codeblock {
    pub code: String,
    pub language: Option<String>,
}

impl Component for Codeblock {
    fn render(&self) -> impl IntoElement {
        let config = use_config();
        let theme = consume_material_theme();

        let ts = consume_root_context::<ThemeSet>();
        let ps = consume_root_context::<SyntaxSet>();

        let code_theme = ts.themes[&config.read().theme.code_theme].clone();

        let deps = use_reactive(&(self.code.clone(), self.language.clone(), code_theme.clone()));

        let spans = use_hook(move || {
            Effect::create_value(move || {
                let (code, lang, theme) = &*deps.read();

                let mut spans = Vec::new();

                if let Some(lang) = lang
                    && let Some(syntax) = ps.find_syntax_by_token(lang)
                {
                    let mut highlight = HighlightLines::new(syntax, theme);

                    for line in LinesWithEndings::from(code) {
                        let ranges = highlight.highlight_line(line, &ps).unwrap();

                        for (style, text) in ranges {
                            spans.push(Span::new(text.to_string()).color(Color::from_argb(
                                style.foreground.a,
                                style.foreground.r,
                                style.foreground.g,
                                style.foreground.b,
                            )));
                        }
                    }
                } else {
                    spans.push(Span::new(code.clone()));
                }

                spans
            })
        });

        let mut selectable_text = SelectableText::new();

        for span in spans.read().iter().cloned() {
            selectable_text = selectable_text.span(span)
        }

        rect().width(Size::Fill).child(
            rect()
                .background(
                    code_theme
                        .settings
                        .background
                        .map(|c| Color::from_argb(c.a, c.r, c.g, c.b))
                        .unwrap_or_else(|| 0xff0d1117.into()),
                )
                .color(
                    code_theme
                        .settings
                        .foreground
                        .map(|c| Color::from_argb(c.a, c.r, c.g, c.b))
                        .unwrap_or_else(|| 0xffc9d1d9.into()),
                )
                .corner_radius(12.)
                .padding(Gaps::new_all(12.))
                .spacing(8.)
                .margin((4., 0.))
                .child(
                    StoatTooltip::new(
                        label()
                            .max_lines(1)
                            .font_size(11.)
                            .text("Copy to clipboard"),
                    )
                    .position(AttachedPosition::Top)
                    .child(
                        StoatButton::new()
                            .corner_radius(3.)
                            .on_press({
                                let code = self.code.clone();
                                move |_| Clipboard::set(code.clone()).unwrap()
                            })
                            .child(
                                rect()
                                    .padding((2., 6.))
                                    .background(theme.md.primary.as_argb_u32())
                                    .color(theme.md.on_primary.as_argb_u32())
                                    .font_size(12.)
                                    .cursor(CursorIcon::Pointer)
                                    .child(
                                        label()
                                            .line_height(1.5)
                                            .font_family("Fira Code Bold")
                                            .text(
                                                self.language
                                                    .as_ref()
                                                    .map(|lang| lang.to_uppercase().into())
                                                    .unwrap_or_else(|| Cow::Borrowed("TXT")),
                                            ),
                                    ),
                            ),
                    ),
                )
                .child(
                    rect().cursor(CursorIcon::Text).child(
                        selectable_text
                            .font_family("Fira Code")
                            .line_height(1.5)
                            .font_size(14.),
                    ),
                ),
        )
    }
}
