use std::{borrow::Cow, mem};

use chumsky::{Parser as _, prelude::*};
use freya::{prelude::*, radio::use_radio};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use stoat_models::v0;

use crate::{
    AppChannel, SizeExt,
    components::{Avatar, material::filled::grid_3x3},
    consume_material_theme, member_display_color, parse_fill,
    theme::Theme,
};

fn consume_server() -> Option<Readable<v0::Server>> {
    consume_context()
}

#[derive(PartialEq)]
pub struct MarkdownViewer {
    content: Cow<'static, str>,

    server: Option<Readable<v0::Server>>,

    layout: LayoutData,
    max_lines: Option<usize>,
    text_overflow: TextOverflow,
    font_size: f32,

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
            max_lines: None,
            text_overflow: TextOverflow::default(),
            font_size: 14.,
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

/// Represents different markdown elements for rendering.
#[allow(dead_code)]
#[derive(Clone)]
enum MarkdownElement {
    Heading {
        level: HeadingLevel,
        content: Vec<Inline>,
    },
    Paragraph {
        content: Vec<Inline>,
    },
    CodeBlock {
        code: String,
        #[allow(dead_code)]
        language: Option<String>,
    },
    UnorderedList {
        items: Vec<Vec<Inline>>,
    },
    OrderedList {
        start: u64,
        items: Vec<Vec<Inline>>,
    },
    Image {
        url: String,
        alt: String,
    },
    Blockquote {
        content: Vec<Inline>,
    },
    Table {
        headers: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    HorizontalRule,
}

/// A piece of a paragraph's content: styled text or an inline link flowing within the text.
#[derive(Clone, Debug, PartialEq)]
enum Inline {
    Span(TextSpan),
    UserMention {
        id: String,
    },
    ChannelMention {
        id: String,
    },
    RoleMention {
        id: String,
    },
    Emoji {
        id: String,
    },
    Link {
        url: String,
        title: Option<String>,
        text: Vec<Inline>,
    },
    Spoiler(Vec<Inline>),
}

/// Represents styled text spans within markdown.
#[derive(Clone, Debug, PartialEq)]
struct TextSpan {
    text: String,
    bold: bool,
    italic: bool,
    strikethrough: bool,
    code: bool,
    link: bool,
    spoiler: bool,
}

impl TextSpan {
    fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            bold: false,
            italic: false,
            strikethrough: false,
            code: false,
            link: false,
            spoiler: false,
        }
    }
}

/// Parse markdown content into a list of elements.
fn parse_markdown(content: &str) -> Vec<MarkdownElement> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(content, options);
    let mut elements = Vec::new();
    let mut current_content: Vec<Inline> = Vec::new();
    let mut list_items: Vec<Vec<Inline>> = Vec::new();
    let mut current_list_item: Vec<Inline> = Vec::new();

    let mut in_heading: Option<HeadingLevel> = None;
    let mut in_paragraph = false;
    let mut in_code_block = false;
    let mut code_block_content = String::new();
    let mut code_block_language: Option<String> = None;
    let mut ordered_list_start: Option<u64> = None;
    let mut in_list_item = false;
    let mut in_blockquote = false;
    let mut blockquote_spans: Vec<Inline> = Vec::new();

    let mut in_table_cell = false;
    let mut table_headers: Vec<Vec<Inline>> = Vec::new();
    let mut table_rows: Vec<Vec<Vec<Inline>>> = Vec::new();
    let mut current_table_row: Vec<Vec<Inline>> = Vec::new();
    let mut current_cell_spans: Vec<Inline> = Vec::new();

    let mut in_link = false;
    let mut link_url: Option<String> = None;
    let mut link_title: Option<String> = None;
    let mut link_spans: Vec<Inline> = Vec::new();

    let mut bold = false;
    let mut italic = false;
    let mut strikethrough = false;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    in_heading = Some(level);
                    current_content.clear();
                }
                Tag::Paragraph => {
                    if in_blockquote {
                        // Paragraphs inside blockquotes
                    } else if in_list_item {
                        // Paragraphs inside list items
                    } else {
                        in_paragraph = true;
                        current_content.clear();
                    }
                }
                Tag::CodeBlock(kind) => {
                    in_code_block = true;
                    code_block_content.clear();
                    code_block_language = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                            let lang_str = lang.to_string();
                            if lang_str.is_empty() {
                                None
                            } else {
                                Some(lang_str)
                            }
                        }
                        pulldown_cmark::CodeBlockKind::Indented => None,
                    };
                }
                Tag::List(start) => {
                    ordered_list_start = start;
                    list_items.clear();
                }
                Tag::Item => {
                    in_list_item = true;
                    current_list_item.clear();
                }
                Tag::Strong => bold = true,
                Tag::Emphasis => italic = true,
                Tag::Strikethrough => strikethrough = true,
                Tag::BlockQuote(_) => {
                    in_blockquote = true;
                    blockquote_spans.clear();
                }
                Tag::Image {
                    dest_url, title, ..
                } => {
                    elements.push(MarkdownElement::Image {
                        url: dest_url.to_string(),
                        alt: title.to_string(),
                    });
                }
                Tag::Link {
                    dest_url, title, ..
                } => {
                    in_link = true;
                    link_url = Some(dest_url.to_string());
                    link_title = Some(title.to_string());
                    link_spans.clear();
                }
                Tag::Table(_) => {
                    table_headers.clear();
                    table_rows.clear();
                    current_table_row.clear();
                }
                Tag::TableHead => {}
                Tag::TableRow => {
                    current_table_row.clear();
                }
                Tag::TableCell => {
                    in_table_cell = true;
                    current_cell_spans.clear();
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    if let Some(level) = in_heading.take() {
                        elements.push(MarkdownElement::Heading {
                            level,
                            content: mem::take(&mut current_content),
                        });
                    }
                }
                TagEnd::Paragraph => {
                    if in_blockquote {
                        blockquote_spans.append(&mut current_content)
                    } else if in_list_item {
                        current_list_item.append(&mut current_content)
                    } else if in_paragraph {
                        in_paragraph = false;
                        elements.push(MarkdownElement::Paragraph {
                            content: mem::take(&mut current_content),
                        });
                    }
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    elements.push(MarkdownElement::CodeBlock {
                        code: mem::take(&mut code_block_content),
                        language: code_block_language.take(),
                    });
                }
                TagEnd::List(_) => {
                    let items = mem::take(&mut list_items);
                    if let Some(start) = ordered_list_start.take() {
                        elements.push(MarkdownElement::OrderedList { start, items });
                    } else {
                        elements.push(MarkdownElement::UnorderedList { items });
                    }
                }
                TagEnd::Item => {
                    in_list_item = false;
                    list_items.push(mem::take(&mut current_list_item));
                }
                TagEnd::Strong => bold = false,
                TagEnd::Emphasis => italic = false,
                TagEnd::Strikethrough => strikethrough = false,
                TagEnd::BlockQuote(_) => {
                    in_blockquote = false;
                    elements.push(MarkdownElement::Blockquote {
                        content: mem::take(&mut blockquote_spans),
                    });
                }
                TagEnd::Table => {
                    elements.push(MarkdownElement::Table {
                        headers: mem::take(&mut table_headers),
                        rows: mem::take(&mut table_rows),
                    });
                }
                TagEnd::TableHead => {
                    // TableHead contains cells directly (no TableRow), so save headers here
                    table_headers = mem::take(&mut current_table_row);
                }
                TagEnd::TableRow => {
                    // TableRow only appears in body rows, not in TableHead
                    table_rows.push(mem::take(&mut current_table_row));
                }
                TagEnd::TableCell => {
                    in_table_cell = false;
                    current_table_row.push(mem::take(&mut current_cell_spans));
                }
                TagEnd::Link => {
                    in_link = false;
                    if let Some(url) = link_url.take() {
                        let title = link_title.take();
                        let text = mem::take(&mut link_spans);

                        if in_table_cell {
                            current_cell_spans.push(Inline::Link { url, title, text });
                        } else if in_blockquote {
                            blockquote_spans.push(Inline::Link { url, title, text });
                        } else if in_list_item {
                            current_list_item.push(Inline::Link { url, title, text });
                        } else if in_link {
                            link_spans.push(Inline::Link { url, title, text });
                        } else {
                            current_content.push(Inline::Link { url, title, text });
                        }

                        // if in_paragraph || in_heading.is_some() {
                        //     current_content.push(Inline::Link { url, title, text });
                        // } else {
                        //     elements.push(MarkdownElement::Link { url, title, text });
                        // }
                    }
                }
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    code_block_content.push_str(text.trim());
                } else if in_table_cell {
                    let text = if let Some(Inline::Span(last)) = current_cell_spans.last()
                        && last.code == false
                        && last.italic == italic
                        && last.bold == bold
                        && last.strikethrough == strikethrough
                    {
                        let output = last.text.to_string() + &*text;
                        current_cell_spans.pop();
                        output
                    } else {
                        text.to_string()
                    };

                    current_cell_spans.extend(parse_elements(&text, bold, italic, strikethrough));
                } else {
                    if in_blockquote && !in_paragraph {
                        let text = if let Some(Inline::Span(last)) = blockquote_spans.last()
                            && last.code == false
                            && last.italic == italic
                            && last.bold == bold
                            && last.strikethrough == strikethrough
                        {
                            let output = last.text.to_string() + &*text;
                            blockquote_spans.pop();
                            output
                        } else {
                            text.to_string()
                        };

                        blockquote_spans.extend(parse_elements(&text, bold, italic, strikethrough));
                    } else if in_list_item && !in_paragraph {
                        let text = if let Some(Inline::Span(last)) = current_list_item.last()
                            && last.code == false
                            && last.italic == italic
                            && last.bold == bold
                            && last.strikethrough == strikethrough
                        {
                            let output = last.text.to_string() + &*text;
                            current_list_item.pop();
                            output
                        } else {
                            text.to_string()
                        };
                        current_list_item.extend(parse_elements(
                            &text,
                            bold,
                            italic,
                            strikethrough,
                        ));
                    } else if in_link {
                        let text = if let Some(Inline::Span(last)) = link_spans.last()
                            && last.code == false
                            && last.italic == italic
                            && last.bold == bold
                            && last.strikethrough == strikethrough
                        {
                            let output = last.text.to_string() + &*text;
                            link_spans.pop();
                            output
                        } else {
                            text.to_string()
                        };
                        link_spans.extend(parse_elements(&text, bold, italic, strikethrough));
                    } else {
                        let text = if let Some(Inline::Span(last)) = current_content.last()
                            && last.code == false
                            && last.italic == italic
                            && last.bold == bold
                            && last.strikethrough == strikethrough
                        {
                            let output = last.text.to_string() + &*text;
                            current_content.pop();
                            output
                        } else {
                            text.to_string()
                        };

                        current_content.extend(parse_elements(&text, bold, italic, strikethrough));
                    }
                }
            }
            Event::Code(code) => {
                let span = Inline::Span(TextSpan {
                    text: code.to_string(),
                    bold,
                    italic,
                    strikethrough,
                    code: true,
                    link: false,
                    spoiler: false,
                });

                if in_table_cell {
                    current_cell_spans.push(span);
                } else if in_blockquote {
                    blockquote_spans.push(span);
                } else if in_list_item {
                    current_list_item.push(span);
                } else if in_link {
                    link_spans.push(span);
                } else {
                    current_content.push(span);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                let span = Inline::Span(TextSpan::new("\n"));
                if in_blockquote {
                    blockquote_spans.push(span);
                } else if in_list_item {
                    current_list_item.push(span);
                } else if in_link {
                    link_spans.push(span);
                } else {
                    current_content.push(span);
                }
            }
            Event::Rule => {
                elements.push(MarkdownElement::HorizontalRule);
            }
            _ => {}
        };
    }

    elements
}

#[derive(Debug, Clone)]
enum InlineParserElement {
    Text(String),
    UserMention(String),
    ChannelMention(String),
    RoleMention(String),
    Emoji(String),
    Link {
        url: String,
        title: Option<String>,
        text: String,
    },
    Hyperlink(String),
    Spoiler(String),
}

fn id_parser<'a>() -> impl chumsky::Parser<'a, &'a str, String> {
    one_of("0123456789ABCDEFGHJKMNPQRSTVWXYZ")
        .repeated()
        .exactly(26)
        .collect()
}

fn user_mention_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    just("@")
        .ignore_then(id_parser())
        .delimited_by(just("<"), just(">"))
        .map(InlineParserElement::UserMention)
}

fn channel_mention_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    just("#")
        .ignore_then(id_parser())
        .delimited_by(just("<"), just(">"))
        .map(InlineParserElement::ChannelMention)
}

fn role_mention_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    just("%")
        .ignore_then(id_parser())
        .delimited_by(just("<"), just(">"))
        .map(InlineParserElement::RoleMention)
}

fn emoji_mention_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    id_parser()
        .delimited_by(just(":"), just(":"))
        .map(InlineParserElement::Emoji)
}

fn link_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    none_of(']')
        .repeated()
        .collect()
        .delimited_by(just("["), just("]"))
        .then(
            none_of(" )")
                .repeated()
                .collect::<String>()
                // .then(
                //     just(' ').repeated().at_least(1).ignore_then(
                //         none_of("\"")
                //             .repeated()
                //             .collect::<String>()
                //             .delimited_by(just("\""), just("\""))
                //             .or_not(),
                //     ),
                // )
                .delimited_by(just('('), just(')')),
        )
        .map(|(text, (url))| InlineParserElement::Link {
            url,
            title: None,
            text,
        })
}

fn spoiler_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    any()
        .and_is(just("||").not())
        .repeated()
        .collect::<String>()
        .padded_by(just("||"))
        .map(InlineParserElement::Spoiler)
}

fn hyperlink_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    any()
        .filter(|c: &char| !c.is_whitespace())
        .repeated()
        .collect::<String>()
        .filter(|text| Url::parse(text).is_ok_and(|url| ["https", "http"].contains(&url.scheme())))
        .map(InlineParserElement::Hyperlink)
}

fn custom_element_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    choice((
        user_mention_parser(),
        channel_mention_parser(),
        role_mention_parser(),
        emoji_mention_parser(),
        link_parser(),
        hyperlink_parser(),
        spoiler_parser(),
    ))
}

fn element_parser<'a>() -> impl chumsky::Parser<'a, &'a str, InlineParserElement> {
    custom_element_parser().or(any::<&str, _>().map(|c| InlineParserElement::Text(c.to_string())))
}

fn inline_parser<'a>() -> impl chumsky::Parser<'a, &'a str, Vec<InlineParserElement>> {
    element_parser().repeated().collect()
}

fn map_element(
    element: InlineParserElement,
    bold: bool,
    italic: bool,
    strikethrough: bool,
) -> Inline {
    match element {
        InlineParserElement::Text(text) => Inline::Span(TextSpan {
            text,
            bold,
            italic,
            strikethrough,
            code: false,
            link: false,
            spoiler: false,
        }),
        InlineParserElement::UserMention(id) => Inline::UserMention { id },
        InlineParserElement::ChannelMention(id) => Inline::ChannelMention { id },
        InlineParserElement::RoleMention(id) => Inline::RoleMention { id },
        InlineParserElement::Emoji(id) => Inline::Emoji { id },
        InlineParserElement::Link { url, title, text } => Inline::Link {
            url,
            title,
            text: parse_elements(&text, bold, italic, strikethrough),
        },
        InlineParserElement::Hyperlink(text) => Inline::Span(TextSpan {
            text,
            bold,
            italic,
            strikethrough,
            code: false,
            link: true,
            spoiler: false,
        }),
        InlineParserElement::Spoiler(text) => {
            Inline::Spoiler(parse_elements(&text, bold, italic, strikethrough))
        }
    }
}

fn parse_elements(content: &str, bold: bool, italic: bool, strikethrough: bool) -> Vec<Inline> {
    let elements = inline_parser()
        .parse(content)
        .into_output()
        .unwrap_or_default()
        .into_iter()
        .fold(Vec::new(), |mut vec, element| {
            match element {
                InlineParserElement::Text(current) => {
                    if matches!(vec.last(), Some(InlineParserElement::Text(_))) {
                        let Some(InlineParserElement::Text(last)) = vec.pop() else {
                            unreachable!()
                        };

                        vec.push(InlineParserElement::Text(last + &current));
                    } else {
                        vec.push(InlineParserElement::Text(current));
                    }
                }
                element => vec.push(element),
            };

            vec
        });

    elements
        .into_iter()
        .map(|element| map_element(element, bold, italic, strikethrough))
        .collect()
}

#[derive(PartialEq)]
struct UserMention {
    pub id: String,
    pub font_size: f32,
}

impl Component for UserMention {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let radio = use_radio(AppChannel::Users);
        let members = radio.slice(AppChannel::Members, |state| &state.members);

        let server = consume_server();

        let state = radio.read();
        let user = state.users.get(&self.id);

        let member = use_memo({
            let id = self.id.clone();
            let server = server.clone();
            let members = members.clone();

            move || {
                if let Some(server) = &server {
                    let server = server.read();
                    let members = members.read();

                    if let Some(members) = members.get(&server.id)
                        && let Some(member) = members.get(&id)
                    {
                        return Some(member.clone());
                    }
                };

                None
            }
        });

        let role_color = use_memo({
            let server = server.clone();

            move || {
                if let Some(server) = &server
                    && let Some(member) = &*member.read()
                {
                    return member_display_color(&*member, &*server.read());
                };

                None
            }
        });

        let username = member
            .read()
            .as_ref()
            .and_then(|m| m.nickname.clone())
            .or(user.map(|u| u.display_name.clone().unwrap_or(u.username.clone())));

        let size = self.font_size * (16. / 14.);

        rect()
            .padding((0., 6., 0., 2.))
            .horizontal()
            .cross_align(Alignment::Center)
            .spacing(4.)
            .corner_radius(16.)
            .background(theme.md.primary_container.as_argb_u32())
            .color(theme.md.on_primary_container.as_argb_u32())
            .font_weight(FontWeight::SEMI_BOLD)
            .maybe_child(
                user.map(|user| {
                    Avatar::new(
                        user.clone().into_readable(),
                        member.read().cloned().map(|m| m.into_readable()),
                        size,
                    )
                })
            )
            .child(
                label()
                .map(role_color.read().cloned(), |label, color| label.color(color))
                    .line_height(1.5)
                    .max_lines(1)
                    .font_size(self.font_size)
                    .text(username.unwrap_or_else(|| "Unknown User".to_string())),
            )
    }
}

#[derive(PartialEq)]
struct ChannelMention {
    pub id: String,
    pub font_size: f32,
}

impl Component for ChannelMention {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let radio = use_radio(AppChannel::Channels);

        let state = radio.read();
        let channel_name = state
            .channels
            .get(&self.id)
            .map(|c| c.name().unwrap_or("<TODO>").to_string());

        let size = self.font_size * (16. / 14.);

        rect()
            .padding((0., 6., 0., 2.))
            .horizontal()
            .cross_align(Alignment::Center)
            .spacing(4.)
            .corner_radius(16.)
            .background(theme.md.primary.as_argb_u32())
            .color(theme.md.on_primary.as_argb_u32())
            .font_weight(FontWeight::SEMI_BOLD)
            .child(SvgViewer::new(grid_3x3()).size(Size::px(size)))
            .child(
                label()
                    .line_height(1.5)
                    .max_lines(1)
                    .font_size(self.font_size)
                    .text(channel_name.unwrap_or_else(|| "Unknown Channel".to_string())),
            )
    }
}

#[derive(PartialEq)]
struct RoleMention {
    pub id: String,
    pub font_size: f32,
}

impl Component for RoleMention {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        let server = consume_server();

        let (name, fill) = use_hook(|| {
            if let Some(server) = server {
                let server = server.read();

                if let Some(role) = server.roles.get(&self.id) {
                    return (
                        role.name.clone(),
                        role.colour.as_deref().and_then(parse_fill),
                    );
                }
            };

            ("Unknown Role".to_string(), None)
        });

        let size = self.font_size * (16. / 14.);
        let fill =
            fill.unwrap_or_else(|| Fill::Color(theme.md.on_primary_container.as_argb_u32().into()));

        rect()
            .color(fill.clone())
            .padding((0., 6., 0., 2.))
            .horizontal()
            .cross_align(Alignment::Center)
            .spacing(4.)
            .corner_radius(16.)
            .background(theme.md.primary_container.as_argb_u32())
            .font_weight(FontWeight::SEMI_BOLD)
            .child(
                rect()
                    .background(fill)
                    .corner_radius(size / 2.)
                    .width(Size::px(size))
                    .height(Size::px(size)),
            )
            .child(
                label()
                    .line_height(1.5)
                    .max_lines(1)
                    .font_size(self.font_size)
                    .text(name),
            )
    }
}

#[derive(PartialEq)]
struct Emoji {
    pub id: String,
    pub font_size: f32,
}

impl Component for Emoji {
    fn render(&self) -> impl IntoElement {
        let size = self.font_size * 1.4;

        rect().padding((0., 0.7, 0., 1.4)).child(
            ImageViewer::new(
                format!("https://cdn.stoatusercontent.com/emojis/{}", &self.id)
                    .parse::<Url>()
                    .unwrap(),
            )
            .sampling_mode(SamplingMode::Trilinear)
            .aspect_ratio(AspectRatio::Max)
            .image_cover(ImageCover::Center)
            .size(Size::px(size)),
        )
    }
}

#[derive(PartialEq)]
struct Link {
    pub content: Vec<Inline>,
    pub url: String,
    pub title: Option<String>,
    pub bold: bool,
    pub font_size: f32,
}

impl Component for Link {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        render_content(
            paragraph(),
            &self.content,
            self.font_size,
            theme,
            self.bold,
            true,
        )
        .on_press(move |_| {})
        .on_pointer_enter(move |_| {
            Cursor::set(CursorIcon::Pointer);
        })
        .on_pointer_leave(move |_| {
            Cursor::set(CursorIcon::default());
        })
    }
}

#[derive(PartialEq)]
struct Spoiler {
    pub content: Vec<Inline>,
    pub font_size: f32,
}

impl Component for Spoiler {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();
        let mut show = use_state(|| false);

        rect()
            .padding((0., 2.))
            .background(
                if show() {
                    theme.md.inverse_surface
                } else {
                    theme.md.surface_container
                }
                .as_argb_u32(),
            )
            .color(
                if show() {
                    theme.md.inverse_on_surface
                } else {
                    theme.md.on_surface
                }
                .as_argb_u32(),
            )
            .corner_radius(12.)
            .on_press(move |_| show.set(true))
            .on_pointer_enter(move |_| {
                Cursor::set(CursorIcon::Pointer);
            })
            .on_pointer_leave(move |_| {
                Cursor::set(CursorIcon::default());
            })
            .child(
                rect()
                    .opacity(if show() { 1. } else { 0. })
                    .child(render_content(
                        paragraph(),
                        &self.content,
                        self.font_size,
                        theme,
                        false,
                        false,
                    )),
            )
    }
}

fn render_content(
    mut p: Paragraph,
    content: &[Inline],
    base_font_size: f32,
    theme: Theme,
    bold: bool,
    link: bool,
) -> Paragraph {
    for item in content {
        p = match item {
            Inline::Span(span) => {
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

                if span.code {
                    p.child(
                        rect()
                            .background(theme.md.surface_container_high.as_argb_u32())
                            .padding((1., 4.))
                            .corner_radius(12.)
                            .child(
                                freya::prelude::paragraph().line_height(1.5).span(styled.font_family("monospace")),
                            ),
                    )
                } else if span.link {
                    p.child(
                        paragraph()
                            .line_height(1.5)
                            .span(styled.color(theme.md.primary.as_argb_u32()))
                            .on_press(move |_| {})
                            .on_pointer_enter(move |_| {
                                Cursor::set(CursorIcon::Pointer);
                            })
                            .on_pointer_leave(move |_| {
                                Cursor::set(CursorIcon::default());
                            }),
                    )
                } else {
                    if link {
                        styled = styled.color(theme.md.primary.as_argb_u32())
                    };

                    p.span(styled)
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

impl Component for MarkdownViewer {
    fn render(&self) -> impl IntoElement {
        let theme = consume_material_theme();

        provide_context(self.server.clone());

        let elements = parse_markdown(&self.content);

        let mut p_container = paragraph()
            .max_lines(self.max_lines)
            .text_overflow(self.text_overflow.clone())
            .line_height(1.5)
            .layout(self.layout.clone())
            .font_size(self.font_size);

        let element_count = elements.len();

        for (idx, element) in elements.into_iter().enumerate() {
            let is_last = idx == element_count - 1;

            match element {
                MarkdownElement::Heading { level, content } => {
                    let multiplier = match level {
                        HeadingLevel::H1 => 2.,
                        HeadingLevel::H2 => 1.6,
                        HeadingLevel::H3 => 1.4,
                        HeadingLevel::H4 => 1.2,
                        HeadingLevel::H5 => 1.,
                        HeadingLevel::H6 => 0.8,
                    };
                    p_container = render_content(
                        p_container,
                        &content,
                        multiplier * self.font_size,
                        theme,
                        true,
                        false,
                    );

                    if !is_last {
                        p_container = p_container.span("\n")
                    };
                }
                MarkdownElement::Paragraph { content } => {
                    p_container =
                        render_content(p_container, &content, self.font_size, theme, false, false);

                    if !is_last {
                        p_container = p_container.span("\n\n")
                    };
                }
                MarkdownElement::CodeBlock { code, .. } => {
                    p_container = p_container.child(
                        rect()
                            .width(Size::fill())
                            .background(theme.md.surface_container_high.as_argb_u32())
                            .color(theme.md.on_surface_variant.as_argb_u32())
                            .corner_radius(6.)
                            .padding(Gaps::new_all(12.))
                            .child(
                                label()
                                    .text(code)
                                    .font_family("monospace")
                                    .font_size(self.font_size),
                            ),
                    )
                }
                MarkdownElement::UnorderedList { items } => {
                    let mut list = rect()
                        .key(idx)
                        .vertical()
                        .spacing(4.)
                        .padding(Gaps::new(0., 0., 0., 20.));

                    for (item_idx, item_spans) in items.into_iter().enumerate() {
                        let item_content = rect()
                            .key(item_idx)
                            .horizontal()
                            .cross_align(Alignment::Start)
                            .spacing(8.)
                            .child(
                                label().text("•").font_size(self.font_size), // .color(theme.md.on_surface.as_argb_u32()),
                            )
                            .child(render_content(
                                paragraph(),
                                &item_spans,
                                self.font_size,
                                theme,
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
                    let mut list = rect()
                        .key(idx)
                        .vertical()
                        .spacing(4.)
                        .padding(Gaps::new(0., 0., 0., 20.));

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
                                    .font_size(self.font_size), // .color(theme.md.on_surface.as_argb_u32()),
                            )
                            .child(render_content(
                                paragraph(),
                                &item_spans,
                                self.font_size,
                                theme,
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
