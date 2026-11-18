use std::mem;

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::components::markdown::parser::parse_elements;

/// Represents different markdown elements for rendering.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum MarkdownElement {
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
    List(List),
    Image {
        url: String,
        alt: String,
    },
    Blockquote {
        content: Vec<MarkdownElement>,
    },
    Table {
        headers: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    HorizontalRule,
}

/// A piece of a paragraph's content: styled text or an inline link flowing within the text.
#[derive(Clone, Debug, PartialEq)]
pub enum Inline {
    Span(TextSpan),
    UserMention {
        id: String,
    },
    ChannelMention {
        id: String,
    },
    MessageMention {
        channel_id: String,
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

/// A markdown list, ordered when `start` is present.
#[derive(Clone, Debug, PartialEq)]
pub struct List {
    pub start: Option<u64>,
    pub items: Vec<ListItem>,
}

/// A list item's inline content plus the lists nested under it.
#[derive(Clone, Debug, PartialEq)]
pub struct ListItem {
    pub content: Vec<Inline>,
    pub nested_lists: Vec<List>,
}

/// Represents styled text spans within markdown.
#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub strikethrough: bool,
    pub code: bool,
    pub link: bool,
    pub spoiler: bool,
}

impl TextSpan {
    pub fn new(text: impl Into<String>) -> Self {
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
pub fn parse_markdown(content: &str) -> Vec<MarkdownElement> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(content, options);
    let mut elements = Vec::new();
    let mut current_content: Vec<Inline> = Vec::new();
    let mut list_stack: Vec<List> = Vec::new();
    let mut item_stack: Vec<ListItem> = Vec::new();

    let mut in_heading: Option<HeadingLevel> = None;
    let mut in_paragraph = false;
    let mut in_code_block = false;
    let mut code_block_content = String::new();
    let mut code_block_language: Option<String> = None;
    let mut blockquotes: Vec<Vec<MarkdownElement>> = Vec::new();

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
                    if !item_stack.is_empty() {
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
                    list_stack.push(List {
                        start,
                        items: Vec::new(),
                    });
                }
                Tag::Item => {
                    item_stack.push(ListItem {
                        content: Vec::new(),
                        nested_lists: Vec::new(),
                    });
                }
                Tag::Strong => bold = true,
                Tag::Emphasis => italic = true,
                Tag::Strikethrough => strikethrough = true,
                Tag::BlockQuote(_) => {
                    blockquotes.push(Vec::new());
                }
                Tag::Image {
                    dest_url, title, ..
                } => {
                    let element = MarkdownElement::Image {
                        url: dest_url.to_string(),
                        alt: title.to_string(),
                    };

                    if let Some(block) = blockquotes.last_mut() {
                        block.push(element);
                    } else {
                        elements.push(element);
                    }
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
                        let element = MarkdownElement::Heading {
                            level,
                            content: mem::take(&mut current_content),
                        };

                        if let Some(block) = blockquotes.last_mut() {
                            block.push(element);
                        } else {
                            elements.push(element);
                        }
                    }
                }
                TagEnd::Paragraph => {
                    if let Some(item) = item_stack.last_mut() {
                        item.content.extend(current_content.drain(..));
                    } else if in_paragraph {
                        in_paragraph = false;
                        let element = MarkdownElement::Paragraph {
                            content: mem::take(&mut current_content),
                        };

                        if let Some(block) = blockquotes.last_mut() {
                            block.push(element);
                        } else {
                            elements.push(element);
                        }
                    }
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;

                    let element = MarkdownElement::CodeBlock {
                        code: mem::take(&mut code_block_content),
                        language: code_block_language.take(),
                    };

                    if let Some(block) = blockquotes.last_mut() {
                        block.push(element);
                    } else {
                        elements.push(element);
                    }
                }
                TagEnd::List(_) => {
                    if let Some(list) = list_stack.pop() {
                        if let Some(item) = item_stack.last_mut() {
                            item.nested_lists.push(list);
                        } else {
                            let element = MarkdownElement::List(list);

                            if let Some(block) = blockquotes.last_mut() {
                                block.push(element);
                            } else {
                                elements.push(element);
                            }
                        }
                    }
                }
                TagEnd::Item => {
                    if let (Some(item), Some(list)) = (item_stack.pop(), list_stack.last_mut()) {
                        list.items.push(item);
                    }
                }
                TagEnd::Strong => bold = false,
                TagEnd::Emphasis => italic = false,
                TagEnd::Strikethrough => strikethrough = false,
                TagEnd::BlockQuote(_) => {
                    let blockquote_spans = blockquotes.pop().unwrap();
                    let element = MarkdownElement::Blockquote {
                        content: blockquote_spans,
                    };

                    if let Some(block) = blockquotes.last_mut() {
                        block.push(element);
                    } else {
                        elements.push(element);
                    }
                }
                TagEnd::Table => {
                    let element = MarkdownElement::Table {
                        headers: mem::take(&mut table_headers),
                        rows: mem::take(&mut table_rows),
                    };

                    if let Some(block) = blockquotes.last_mut() {
                        block.push(element);
                    } else {
                        elements.push(element);
                    }
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
                        let element = Inline::Link { url, title, text };

                        if in_table_cell {
                            current_cell_spans.push(element);
                        } else if let Some(item) = item_stack.last_mut() {
                            item.content.push(element);
                        } else if in_link {
                            link_spans.push(element);
                        } else {
                            current_content.push(element);
                        }
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
                    if let Some(item) = item_stack.last_mut()
                        && !in_paragraph
                    {
                        let text = if let Some(Inline::Span(last)) = item.content.last()
                            && last.code == false
                            && last.italic == italic
                            && last.bold == bold
                            && last.strikethrough == strikethrough
                        {
                            let output = last.text.to_string() + &*text;
                            item.content.pop();
                            output
                        } else {
                            text.to_string()
                        };
                        item.content
                            .extend(parse_elements(&text, bold, italic, strikethrough));
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
                } else if let Some(item) = item_stack.last_mut() {
                    item.content.push(span);
                } else if in_link {
                    link_spans.push(span);
                } else {
                    current_content.push(span);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                let span = Inline::Span(TextSpan::new("\n"));

                if let Some(item) = item_stack.last_mut() {
                    item.content.push(span);
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
