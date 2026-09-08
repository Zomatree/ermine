use std::borrow::Cow;

use freya::prelude::*;
use freya_core::{element::EventHandlerType, integration::EventName};
use pulldown_cmark::HeadingLevel;
use rustc_hash::FxHashMap;
use stoat_models::v0;

use crate::{
    components::markdown::{
        components::Codeblock,
        parser::{MarkdownElement, parse_markdown},
        render::render_content,
    },
    consume_material_theme,
};

#[derive(PartialEq)]
pub struct MarkdownViewer {
    content: Cow<'static, str>,

    server: Option<Readable<v0::Server>>,

    layout: LayoutData,
    accessibility: AccessibilityData,
    max_lines: Option<usize>,
    text_overflow: TextOverflow,
    font_size: f32,

    cursor_index: Option<usize>,
    cursor_style: CursorStyle,
    highlights: Vec<(usize, usize)>,
    holder: ParagraphHolder,
    event_handlers: FxHashMap<EventName, EventHandlerType>,

    key: DiffKey,
}

impl MarkdownViewer {
    pub fn new(
        content: impl Into<Cow<'static, str>>,
        server: Option<Readable<v0::Server>>,
    ) -> Self {
        Self {
            content: content.into(),

            server,

            layout: LayoutData::default(),
            accessibility: AccessibilityData::default(),
            max_lines: None,
            text_overflow: TextOverflow::default(),
            font_size: 14.,

            cursor_index: None,
            cursor_style: CursorStyle::Line,
            highlights: Vec::new(),
            holder: ParagraphHolder::default(),
            event_handlers: FxHashMap::default(),

            key: DiffKey::None,
        }
    }

    pub fn font_size(mut self, font_size: impl Into<FontSize>) -> Self {
        self.font_size = *font_size.into() as f32;

        self
    }

    pub fn max_lines(mut self, max_lines: impl Into<Option<usize>>) -> Self {
        self.max_lines = max_lines.into();
        self
    }

    pub fn text_overflow(mut self, text_overflow: impl Into<TextOverflow>) -> Self {
        self.text_overflow = text_overflow.into();
        self
    }

    pub fn cursor_style(mut self, cursor_style: impl Into<CursorStyle>) -> Self {
        self.cursor_style = cursor_style.into();
        self
    }

    pub fn holder(mut self, holder: ParagraphHolder) -> Self {
        self.holder = holder;
        self
    }

    pub fn cursor_index(mut self, cursor_index: impl Into<Option<usize>>) -> Self {
        self.cursor_index = cursor_index.into();
        self
    }

    pub fn highlights(mut self, highlights: impl Into<Option<Vec<(usize, usize)>>>) -> Self {
        if let Some(highlights) = highlights.into() {
            self.highlights = highlights;
        }
        self
    }
}

impl KeyExt for MarkdownViewer {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl LayoutExt for MarkdownViewer {
    fn get_layout(&mut self) -> &mut LayoutData {
        &mut self.layout
    }
}

impl ContainerExt for MarkdownViewer {}

impl AccessibilityExt for MarkdownViewer {
    fn get_accessibility_data(&mut self) -> &mut AccessibilityData {
        &mut self.accessibility
    }
}

impl EventHandlersExt for MarkdownViewer {
    fn get_event_handlers(&mut self) -> &mut FxHashMap<EventName, EventHandlerType> {
        &mut self.event_handlers
    }
}

impl Component for MarkdownViewer {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        provide_context(self.server.clone());

        let elements = parse_markdown(&self.content);

        let mut p_container = paragraph()
            .layout(self.layout.clone())
            .accessibility(self.accessibility.clone())
            .max_lines(self.max_lines)
            .text_overflow(self.text_overflow.clone())
            .line_height(1.5)
            .font_size(self.font_size)
            .cursor_index(self.cursor_index)
            .cursor_style(self.cursor_style)
            .highlights(self.highlights.clone())
            .holder(self.holder.clone())
            .event_handlers(self.event_handlers.clone())
            .cursor_color(0xFFFFFFFF);

        let element_count = elements.len();

        for (idx, element) in elements.into_iter().enumerate() {
            let is_last = idx == element_count - 1;

            match element {
                MarkdownElement::Heading { level, content } => {
                    let multiplier = match level {
                        HeadingLevel::H1 => 2.0,
                        HeadingLevel::H2 => 1.6,
                        HeadingLevel::H3 => 1.4,
                        HeadingLevel::H4 => 1.2,
                        HeadingLevel::H5 => 1.0,
                        HeadingLevel::H6 => 0.8,
                    };
                    p_container = render_content(
                        p_container,
                        &content,
                        (multiplier * self.font_size).round(),
                        theme,
                        true,
                        false,
                        false,
                    );

                    if !is_last {
                        p_container = p_container.span("\n")
                    };
                }
                MarkdownElement::Paragraph { content } => {
                    p_container = render_content(
                        p_container,
                        &content,
                        self.font_size,
                        theme,
                        false,
                        false,
                        false,
                    );

                    if !is_last {
                        p_container = p_container.span("\n\n")
                    };
                }
                MarkdownElement::CodeBlock { code, language } => {
                    p_container = p_container.child(Codeblock { code, language })
                }
                MarkdownElement::UnorderedList { items } => {
                    let mut list = rect().key(idx).vertical().spacing(4.);
                    // .padding(Gaps::new(0., 0., 0., 20.));

                    for (item_idx, item_spans) in items.into_iter().enumerate() {
                        let item_content = rect()
                            .key(item_idx)
                            .horizontal()
                            .cross_align(Alignment::Start)
                            .spacing(8.)
                            .child(
                                label().text("•").line_height(1.5).font_size(self.font_size), // .color(theme.md.on_surface.as_argb_u32()),
                            )
                            .child(render_content(
                                paragraph(),
                                &item_spans,
                                self.font_size,
                                theme,
                                false,
                                false,
                                false,
                            ));

                        list = list.child(item_content);
                    }

                    p_container = p_container.child(list);

                    if !is_last {
                        p_container = p_container.span("\n\n")
                    };
                }
                MarkdownElement::OrderedList { start, items } => {
                    let mut list = rect().key(idx).vertical().spacing(4.);
                    // .padding(Gaps::new(0., 0., 0., 20.));

                    for (item_idx, item_spans) in items.into_iter().enumerate() {
                        let number = start + item_idx as u64;
                        let item_content = rect()
                            .key(item_idx)
                            .horizontal()
                            .cross_align(Alignment::Start)
                            .spacing(8.)
                            .child(
                                label()
                                    .text(format!("{}.", number))
                                    .line_height(1.5)
                                    .font_size(self.font_size), // .color(theme.md.on_surface.as_argb_u32()),
                            )
                            .child(render_content(
                                paragraph(),
                                &item_spans,
                                self.font_size,
                                theme,
                                false,
                                false,
                                false,
                            ));

                        list = list.child(item_content);
                    }

                    p_container = p_container.child(list);

                    if !is_last {
                        p_container = p_container.span("\n\n")
                    };
                }
                MarkdownElement::Image { alt, .. } => {
                    p_container = p_container.span(format!("[Image: {}]", alt))
                }
                MarkdownElement::Blockquote { content: spans } => {
                    p_container = p_container.child(
                        rect()
                            .width(Size::fill())
                            .padding((4., 12.))
                            .corner_radius(8.)
                            .border(
                                Border::new()
                                    .width(BorderWidth {
                                        top: 0.,
                                        right: 0.,
                                        left: 4.,
                                        bottom: 0.,
                                    })
                                    .fill(theme.md.secondary.as_argb_u32())
                                    .alignment(BorderAlignment::Inner),
                            )
                            .background(theme.md.secondary_container.as_argb_u32())
                            .child(
                                render_content(
                                    paragraph(),
                                    &spans,
                                    self.font_size,
                                    theme,
                                    false,
                                    false,
                                    false,
                                )
                                .font_slant(FontSlant::Italic),
                            ),
                    )
                }
                MarkdownElement::HorizontalRule => {
                    p_container = p_container.child(
                        rect()
                            .width(Size::fill())
                            .height(Size::px(1.))
                            .background(theme.md.on_surface.as_argb_u32()),
                    )
                }
                MarkdownElement::Table { headers, rows } => {
                    let mut head = TableHead::new();
                    let mut header_row = TableRow::new();
                    let column_count = headers.len();

                    for (col_idx, header_spans) in headers.into_iter().enumerate() {
                        header_row = header_row.child(
                            TableCell::new()
                                .key(col_idx)
                                .height(Size::Inner)
                                .padding((8.).into())
                                .child(
                                    render_content(
                                        paragraph(),
                                        &header_spans,
                                        self.font_size,
                                        theme,
                                        true,
                                        false,
                                        false,
                                    )
                                    .width(Size::Fill)
                                    .text_align(TextAlign::Start),
                                ),
                        );
                    }
                    head = head.child(header_row);

                    let mut body = TableBody::new();
                    for (row_idx, row) in rows.into_iter().enumerate() {
                        let mut table_row = TableRow::new().key(row_idx);
                        for (col_idx, cell_spans) in row.into_iter().enumerate() {
                            table_row = table_row.child(
                                rect()
                                    .key(col_idx)
                                    .overflow(Overflow::Clip)
                                    .width(Size::fill())
                                    .main_align(Alignment::Start)
                                    .cross_align(Alignment::Center)
                                    .min_height(Size::px(35.))
                                    .padding((0., 8.))
                                    .horizontal()
                                    .child(
                                        render_content(
                                            paragraph(),
                                            &cell_spans,
                                            self.font_size,
                                            theme,
                                            false,
                                            false,
                                            false,
                                        )
                                        .width(Size::Fill)
                                        .text_align(TextAlign::Start),
                                    ),
                            );
                        }
                        body = body.child(table_row);
                    }

                    p_container = p_container.child(
                        Table::new()
                            .column_widths(vec![Size::flex(1.); column_count])
                            .key(idx)
                            .child(head)
                            .child(body),
                    )
                }
            };
        }

        p_container
    }

    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }
}
