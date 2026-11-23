use std::hash::{DefaultHasher, Hash, Hasher};

use freya::prelude::*;
use latex_rust::{
    Color as LatexColor, Dim, MathFont, MathStyle, ParseOptions, SvgOptions,
    layout_with_em_size_pt, parse_with_options, render_svg_with_metrics,
};

use crate::{
    components::{StoatTooltip, markdown::MarkdownColours},
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct Latex {
    pub content: String,
    pub font_size: f32,
    pub colours: MarkdownColours,
}

impl Component for Latex {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let content = use_reactive(&self.content);
        let colours = use_reactive(&self.colours);
        let font_size = self.font_size;

        let svg = use_derived::<Result<_, String>>(move || {
            let colours = colours.read();
            let content = content.read();
            let font_size = Dim::from_i64((font_size * 1.2) as i64);

            let (ast, _) =
                parse_with_options(&*content, &ParseOptions::new()).map_err(|e| e.to_string())?;

            let font = MathFont::stix_two_math().map_err(|e| e.to_string())?;
            let tree = layout_with_em_size_pt(&ast, &font, MathStyle::Text, &font_size)
                .map_err(|e| e.to_string())?;

            let svg = render_svg_with_metrics(
                &tree,
                &font,
                &SvgOptions {
                    font_size_pt: font_size,
                    color: LatexColor::rgb(
                        colours.foreground.r(),
                        colours.foreground.g(),
                        colours.foreground.b(),
                    ),
                    display: false,
                },
            )
            .map_err(|e| e.to_string())?;

            let mut hasher = DefaultHasher::default();
            svg.svg.hash(&mut hasher);

            let width = svg.width_pt.ceil_to_u32().unwrap_or(1);
            let height = svg.height_pt.ceil_to_u32().unwrap_or(1);

            Ok((hasher.finish(), width, height, Bytes::from(svg.svg)))
        });

        match svg.read().cloned() {
            Ok((hash, width, height, svg)) => SvgViewer::new(ImageSource::Bytes(hash, svg))
                .width(Size::px(width as f32))
                .height(Size::px(height as f32))
                .into_element(),
            Err(e) => StoatTooltip::new(label().font_size(11).max_lines(1).text(e))
                .child(
                    label()
                        .color(theme.md.error)
                        .font_size(font_size)
                        .text(self.content.clone()),
                )
                .position(AttachedPosition::Top)
                .into_element(),
        }
    }
}
